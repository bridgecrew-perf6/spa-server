use hyper::server::Server as HServer;
use socket2::{Domain, Socket, Type};
use std::convert::Infallible;
use std::future::Future;
use std::net::{SocketAddr, TcpListener};

use hyper::server::conn::AddrIncoming;
use std::str::FromStr;
use std::sync::Arc;
use chrono::Local;
use tokio::net::TcpListener as TKTcpListener;
use tokio::sync::oneshot::Receiver;
use warp::cors::Cors;
use warp::Filter;

use crate::config::Config;
use crate::domain_storage::DomainStorage;
use crate::join;
use crate::redirect_https::hyper_redirect_server;
use crate::static_file_filter::static_file_filter;
use crate::tls::{get_tls_config, TlsAcceptor};

pub struct Server {
    conf: Config,
    storage: Arc<DomainStorage>,

}

impl Server {
    pub fn new(conf: Config, storage: Arc<DomainStorage>) -> Self {
        Server { conf, storage }
    }

    fn get_cors_config(&self) -> Cors {
        warp::cors()
            .allow_any_origin()
            .allow_methods(vec!["GET", "OPTION", "HEAD"])
            .max_age(3600)
            .build()
    }

    async fn start_https_server<F>(&self, func:F) -> anyhow::Result<()>
        where F: Future<Output = ()>  {
        if let Some(config) = &self.conf.https {
            let tls_server_config = get_tls_config(&config.public, &config.private)?;
            let bind_address =
                SocketAddr::from_str(&format!("{}:{}", &config.addr, &config.port)).unwrap();
            let filter = static_file_filter(self.storage.clone());
            tracing::info!("listenins on https://{}", &bind_address);
            let incoming =
                AddrIncoming::from_listener(TKTcpListener::from_std(get_socket(bind_address)?)?)?;
            if self.conf.cors {
                tracing::debug!("enable CORS for https server");
                let service = warp::service(filter.with(self.get_cors_config()));
                let make_svc = hyper::service::make_service_fn(|_| {
                    let service = service.clone();
                    async move { Ok::<_, Infallible>(service) }
                });

                HServer::builder(TlsAcceptor::new(tls_server_config, incoming))
                    .serve(make_svc).with_graceful_shutdown(func)
                    .await?;
                // warp::serve(filter.with(self.get_cors_config()))
                //     .tls()
                //     .cert_path(&config.public)
                //     .key_path(&config.private)
                //     .run(bind_address)
                //     .await;
            } else {
                let service = warp::service(filter);
                let make_svc = hyper::service::make_service_fn(|_| {
                    let service = service.clone();
                    async move { Ok::<_, Infallible>(service) }
                });
                HServer::builder(TlsAcceptor::new(tls_server_config, incoming))
                    .serve(make_svc).with_graceful_shutdown(func)
                    .await?;
                // warp::serve(filter)
                //     .tls()
                //     .cert_path(&config.public)
                //     .key_path(&config.private)
                //     .run(bind_address)
                //     .await;
            }
        }
        Ok(())
    }

    async fn start_http_server<F>(&self, func:F) -> anyhow::Result<()>
    where  F: Future<Output = ()>  {
        if self.conf.port > 0 {
            let bind_address =
                SocketAddr::from_str(&format!("{}:{}", &self.conf.addr, &self.conf.port)).unwrap();
            if self
                .conf
                .https
                .as_ref()
                .map_or(false, |x| x.http_redirect_to_https)
            {
                hyper_redirect_server(bind_address, self.storage.clone()).await?;
            } else {
                let filter = static_file_filter(self.storage.clone());
                if self.conf.cors {
                    tracing::debug!("enable CORS for http server");
                    let service = warp::service(filter.with(self.get_cors_config()));
                    let make_svc = hyper::service::make_service_fn(|_| {
                        let service = service.clone();
                        async move { Ok::<_, Infallible>(service) }
                    });
                    tracing::info!("listening on http://{}", &bind_address);
                    HServer::from_tcp(get_socket(bind_address)?)?
                        .serve(make_svc).with_graceful_shutdown(func)
                        .await?;
                    // warp::serve(filter.with(self.get_cors_config()))
                    //     .run(bind_address)
                    //     .await;
                } else {
                    let service = warp::service(filter);
                    let make_svc = hyper::service::make_service_fn(|_| {
                        let service = service.clone();
                        async move { Ok::<_, Infallible>(service) }
                    });
                    tracing::info!("listening on http://{}", &bind_address);
                    HServer::from_tcp(get_socket(bind_address)?)?
                        .serve(make_svc).with_graceful_shutdown(func)
                        .await?;
                    //warp::serve(filter).run(bind_address).await;
                }
            }
        }
        Ok(())
    }
    pub async fn run(&self, http_rx:Option<Receiver<()>>, https_rx:Option<Receiver<()>>) -> anyhow::Result<()> {
        //disable http server
        if self.conf.port <= 0 && self.conf.https.is_none() {
            panic!("should set http or https server config");
        }
        let time = Local::now();
        let _re = join(self.start_http_server(async move {
            if let Some(rx) = http_rx {
                rx.await.ok();
                tracing::info!("prepare to close http server which start at {}", time.format("%Y-%m-%d %H:%M:%S"));
            }
        }), self.start_https_server(async move {
            if let Some(rx) = https_rx {
                rx.await.ok();
                tracing::info!("prepare to close https server which start at {}", time.format("%Y-%m-%d %H:%M:%S"));
            }
        })).await;
        _re.0?;
        _re.1?;
        Ok(())
    }
}

pub fn get_socket(address: SocketAddr) -> anyhow::Result<TcpListener> {
    let socket = Socket::new(Domain::for_address(address), Type::STREAM, None)?;
    socket.set_nodelay(true)?;
    // socket.set_reuse_address(true)?;
    socket.set_reuse_port(true)?;
    socket.set_nonblocking(true)?;
    #[cfg(any(target_os = "linux", target_vendor = "apple"))]
    socket.bind(&address.into())?;
    socket.listen(128)?;
    let listener: TcpListener = socket.into();
    Ok(listener)
}
