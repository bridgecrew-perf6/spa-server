use crate::file_cache::{CacheItem, FileCache};
use anyhow::anyhow;
use bytes::Bytes;
use dashmap::DashMap;
use lazy_static::lazy_static;
use md5::{Digest, Md5};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::{DirEntry, WalkDir};
use warp::fs::sanitize_path;

pub(crate) const URI_REGEX_STR: &str =
    "[a-zA-Z0-9][-a-zA-Z0-9]{0,62}(\\.[a-zA-Z0-9][-a-zA-Z0-9]{0,62})+\\.?";
//"[a-zA-Z0-9][-a-zA-Z0-9]{0,62}(\\.[a-zA-Z0-9][-a-zA-Z0-9]{0,62})+$";

lazy_static! {
    pub static ref URI_REGEX: Regex = Regex::new(URI_REGEX_STR).unwrap();
}

pub(crate) const UPLOADING_FILE_NAME: &str = ".SPA-Processing";

pub struct DomainStorage {
    meta: DashMap<String, (PathBuf, u32)>,
    prefix: PathBuf,
    cache: FileCache,
    uploading_status: DashMap<String, u32>,
}

impl DomainStorage {
    pub fn init<T: AsRef<Path>>(path_prefix: T, cache: FileCache) -> anyhow::Result<DomainStorage> {
        let path_prefix = path_prefix.as_ref();
        let path_prefix_buf = path_prefix.to_path_buf();
        if path_prefix.exists() {
            let domain_version: DashMap<String, (PathBuf, u32)> = DashMap::new();
            let uploading_status: DashMap<String, u32> = DashMap::new();

            let domain_dirs = fs::read_dir(path_prefix)?;
            for domain_dir in domain_dirs {
                let domain_dir = domain_dir?;
                let metadata = domain_dir.metadata()?;
                let domain_dir_name = domain_dir.file_name();
                let domain_dir_name = domain_dir_name.to_str().unwrap();

                if metadata.is_dir() && URI_REGEX.is_match(domain_dir_name) {
                    let mut max_version = 0;
                    for version_dir_entry in fs::read_dir(domain_dir.path())? {
                        let version_dir_entry = version_dir_entry?;
                        if let Some(version_dir) = version_dir_entry
                            .file_name()
                            .to_str()
                            .map(|file_name| file_name.parse::<u32>().ok())
                            .flatten()
                        {
                            let mut path = version_dir_entry.path();
                            path.push(UPLOADING_FILE_NAME);
                            // this directory is in uploading
                            if path.exists() {
                                uploading_status.insert(domain_dir_name.to_string(), version_dir);
                            } else if max_version < version_dir {
                                max_version = version_dir
                            }
                        }
                    }
                    if max_version > 0 {
                        tracing::info!(
                            "serve domain: {},version: {}",
                            domain_dir_name,
                            max_version
                        );
                        let path_buf = path_prefix_buf
                            .join(domain_dir_name)
                            .join(max_version.to_string());
                        let data = cache.cache_dir(&path_buf)?;
                        cache.update(domain_dir_name.to_string(), data);
                        domain_version.insert(domain_dir_name.to_owned(), (path_buf, max_version));
                    }
                }
            }
            Ok(DomainStorage {
                meta: domain_version,
                prefix: path_prefix.to_path_buf(),
                cache,
                uploading_status,
            })
        } else {
            Err(anyhow!("{:?} does not exist", path_prefix))
        }
    }

    pub fn get_file(&self, host: &str, key: &str) -> Option<Arc<CacheItem>> {
        self.cache.get_item(host, key)
    }

    pub async fn upload_domain_with_version(
        &self,
        domain: String,
        version: u32,
    ) -> anyhow::Result<()> {
        let new_path = self.prefix.join(&domain).join(version.to_string());
        if self
            .uploading_status
            .get(&domain)
            .filter(|x| *x.value() == version)
            .is_some()
        {
            Err(anyhow!(
                "domain:{},version:{} is uploading now,please to change upload status",
                domain,
                version
            ))
        } else if new_path.is_dir() {
            self.meta
                .insert(domain.clone(), (new_path.clone(), version));
            let data = self.cache.cache_dir(&new_path)?;
            tracing::info!("update domain:{}, version:{} ", &domain, version);
            self.cache.update(domain, data);
            Ok(())
        } else {
            Err(anyhow!("{:?} does not exits", new_path))
        }
    }

    fn get_version_path(&self, host: &str, version: u32) -> PathBuf {
        let mut prefix = self.prefix.clone();
        prefix.push(host);
        prefix.push(version.to_string());
        prefix
    }

    pub fn get_new_upload_path(&self, domain: &str) -> PathBuf {
        if let Some(version) = self.uploading_status.get(domain).map(|x| *x.value()) {
            self.get_version_path(domain, version)
        } else {
            match self.get_domain_info_by_domain(domain) {
                Some(domain_info) => {
                    let max_version = domain_info.versions.iter().max().unwrap_or(&0);
                    self.prefix.join(domain).join((max_version + 1).to_string())
                }
                None => self.prefix.join(domain).join(1.to_string()),
            }
        }
    }

    pub fn get_domain_info_by_domain(&self, domain: &str) -> Option<DomainInfo> {
        self.get_domain_info()
            .into_iter()
            .find(|x| x.domain == domain)
    }

    pub fn get_domain_info(&self) -> Vec<DomainInfo> {
        let mut result: Vec<DomainInfo> = Vec::new();
        for item in self.meta.iter() {
            let (path, version) = item.value();
            let mut versions: Vec<i32> = Vec::new();
            if let Ok(version_dir) = fs::read_dir(path.parent().unwrap()) {
                for version in version_dir {
                    if let Ok(version) = version {
                        if let Some(Ok(version)) =
                            version.file_name().to_str().map(|x| x.parse::<i32>())
                        {
                            versions.push(version)
                        }
                    }
                }
            }
            result.push(DomainInfo {
                domain: item.key().to_string(),
                current_version: *version,
                versions,
            })
        }
        result
    }
    fn check_is_in_upload_process(&self, domain: &str, version: &u32) -> bool {
        self.uploading_status
            .get(domain)
            .filter(|x| x.value() == version)
            .is_some()
    }

    pub fn get_files_metadata(
        &self,
        domain: String,
        version: u32,
    ) -> anyhow::Result<Vec<ShortMetaData>> {
        let path_buf = self.get_version_path(&domain, version);
        if path_buf.exists() {
            let prefix = path_buf
                .to_str()
                .map(|x| Ok(format!("{}/", x.to_string())))
                .unwrap_or(Err(anyhow!("can not parse path")))?;
            let mut byte_buffer = vec![0u8; 1024 * 1024];

            fn get_short_metadata(
                entry: DirEntry,
                prefix: &str,
                byte_buffer: &mut Vec<u8>,
            ) -> Option<ShortMetaData> {
                let x = entry.path().to_str()?;
                let key = x.replace(prefix, "");
                if let Ok(meta) = entry.metadata() {
                    let md5 = File::open(entry.path())
                        .ok()
                        .map(|mut f| {
                            let mut hasher = Md5::new();
                            //if file_size > 1024 * 1024 {
                            //1Mb
                            loop {
                                let n = f.read(byte_buffer).ok()?;
                                let valid_buf_slice = &byte_buffer[..n];
                                if n == 0 {
                                    break;
                                }
                                hasher.update(valid_buf_slice);
                            }
                            Some(format!("{:x}", hasher.finalize()))
                        })
                        .flatten()?;
                    let ret = ShortMetaData {
                        path: key,
                        md5: md5,
                        length: meta.len(),
                    };
                    tracing::trace!("ShortMetaData {:?}", ret);
                    Some(ret)
                } else {
                    None
                }
            }
            let ret = WalkDir::new(&path_buf)
                .into_iter()
                .filter_map(|x| x.ok())
                .filter(|x| x.file_name() != UPLOADING_FILE_NAME && x.file_type().is_file())
                .filter_map(|entry| get_short_metadata(entry, &prefix, &mut byte_buffer))
                .collect::<Vec<ShortMetaData>>();
            Ok(ret)
        } else {
            Err(anyhow!("the path does not exists"))
        }
    }

    pub fn save_file(
        &self,
        domain: String,
        version: u32,
        path: String,
        data: Bytes,
    ) -> anyhow::Result<()> {
        if self.check_is_in_upload_process(&domain, &version) {
            let file_path = sanitize_path(self.get_version_path(&domain, version), &path)
                .map_err(|_| anyhow!("path error"))?;
            let mut file = if !file_path.exists() {
                File::create(file_path)?
            } else {
                File::open(file_path)?
            };
            file.write_all(&data)?;
            Ok(())
        } else {
            Err(anyhow!(
                "domain:{}, version:{} can't be uploaded file, it's not in the status allowing to upload file",
                domain,
                version,
            ))
        }
    }

    pub fn update_uploading_status(
        &self,
        domain: String,
        version: u32,
        uploading_status: UploadingStatus,
    ) -> anyhow::Result<()> {
        if self
            .get_domain_info_by_domain(&domain)
            .filter(|x| x.current_version != version)
            .is_none()
        {
            if let Some(uploading_version) = self.uploading_status.get(&domain).map(|v| *v.value())
            {
                if uploading_version != version {
                    return Err(anyhow!(
                        "domain:{}, version:{} is in uploading, please finish it firstly",
                        domain,
                        uploading_version,
                    ));
                } else if uploading_status == UploadingStatus::Finish {
                    let mut p = self.get_version_path(&domain, version);
                    p.push(UPLOADING_FILE_NAME);
                    fs::remove_file(p)?;
                    self.uploading_status
                        .remove_if(&domain, |_, v| *v == version);
                    tracing::info!(
                        "domain:{}, version:{} change to upload status:finish",
                        domain,
                        version
                    );
                }
            } else if uploading_status == UploadingStatus::Uploading {
                let mut p = self.get_version_path(&domain, version);
                p.push(UPLOADING_FILE_NAME);
                tracing::info!(
                    "domain:{}, version:{} change to upload status:uploading",
                    domain,
                    version
                );
                self.uploading_status.insert(domain, version);
                File::create(p)?;
            }
            Ok(())
        } else {
            Err(anyhow!(
                "domain:{}, version:{} is in serving, can not change upload status",
                domain,
                version,
            ))
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DomainInfo {
    pub domain: String,
    pub current_version: u32,
    pub versions: Vec<i32>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ShortMetaData {
    pub path: String,
    pub md5: String,
    pub length: u64,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum UploadingStatus {
    Uploading = 0,
    Finish = 1,
}

#[cfg(test)]
mod test {
    use crate::domain_storage::URI_REGEX_STR;
    use regex::Regex;

    #[test]
    fn test_uri_regex() {
        let uri_regex = Regex::new(URI_REGEX_STR).unwrap();
        // println!("{}", uri_regex.is_match("www.baidu.com"));
        assert!(
            uri_regex.is_match("www.baidu.com"),
            "uri_regex can not parse www.baidu.com"
        );
        assert!(!uri_regex.is_match("baidu"));
        //println!("{}", uri_regex.is_match("abc"));
        //assert!(!uri_regex.is_match("abc"));
        let r2 = Regex::new("\\.").unwrap();
        assert!(r2.is_match("."));
        assert!(!r2.is_match("x"));
    }
}
