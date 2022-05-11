import{_ as t,c as e,o as a,a as d}from"./app.33b64e60.js";const g='{"title":"NPM Package","description":"","frontmatter":{},"headers":[{"level":2,"title":"Overview","slug":"overview"},{"level":2,"title":"Install in new project","slug":"install-in-new-project"},{"level":2,"title":"Operating Systems","slug":"operating-systems"},{"level":2,"title":"Source Code","slug":"source-code"},{"level":3,"title":"Command Line","slug":"command-line"}],"relativePath":"guide/spa-client-npm-package.md"}',n={},r=d(`<h1 id="npm-package" tabindex="-1">NPM Package <a class="header-anchor" href="#npm-package" aria-hidden="true">#</a></h1><h2 id="overview" tabindex="-1">Overview <a class="header-anchor" href="#overview" aria-hidden="true">#</a></h2><p>the npm package wraps spa-client command line by <a href="https://github.com/napi-rs/napi-rs" target="_blank" rel="noopener noreferrer">napi-rs</a>, like <a href="https://github.com/swc-project/swc" target="_blank" rel="noopener noreferrer">swc</a>. So both have same user experience and same api.</p><p>There has an example project for npm package users: <a href="https://github.com/timzaak/spa-server/tree/master/example/js-app-example" target="_blank" rel="noopener noreferrer">js-app-example</a>.</p><h2 id="install-in-new-project" tabindex="-1">Install in new project <a class="header-anchor" href="#install-in-new-project" aria-hidden="true">#</a></h2><p>there is more info at <a href="./getting-started.html#run-spa-client-in-npm-package">getting started</a></p><h2 id="operating-systems" tabindex="-1">Operating Systems <a class="header-anchor" href="#operating-systems" aria-hidden="true">#</a></h2><table><thead><tr><th></th><th>node12</th><th>node14</th><th>node16</th></tr></thead><tbody><tr><td>Windows x64</td><td>\u2713</td><td>\u2713</td><td>\u2713</td></tr><tr><td>Windows x32</td><td>\u2713</td><td>\u2713</td><td>\u2713</td></tr><tr><td>Windows arm64</td><td>x</td><td>x</td><td>x</td></tr><tr><td>macOS x64</td><td>\u2713</td><td>\u2713</td><td>\u2713</td></tr><tr><td>macOS arm64</td><td>\u2713</td><td>\u2713</td><td>\u2713</td></tr><tr><td>Linux x64 gnu</td><td>x</td><td>x</td><td>\u2713</td></tr><tr><td>Linux x64 musl</td><td>\u2713</td><td>\u2713</td><td>\u2713</td></tr><tr><td>Linux arm gnu</td><td>\u2713</td><td>\u2713</td><td>\u2713</td></tr><tr><td>Linux arm64 gnu</td><td>\u2713</td><td>\u2713</td><td>\u2713</td></tr><tr><td>Linux arm64 musl</td><td>\u2713</td><td>\u2713</td><td>\u2713</td></tr><tr><td>Android arm64</td><td>\u2713</td><td>\u2713</td><td>\u2713</td></tr><tr><td>Android armv7</td><td>\u2713</td><td>\u2713</td><td>\u2713</td></tr><tr><td>FreeBSD x64</td><td>\u2713</td><td>\u2713</td><td>\u2713</td></tr></tbody></table><p>Windows arm64 blocked by: <a href="https://github.com/briansmith/ring/issues/1167" target="_blank" rel="noopener noreferrer">https://github.com/briansmith/ring/issues/1167</a></p><p>Linux x64 gnu(node12,node14) blocked by : <code>Error: /build/jsclient/spa-client.linux-x64-gnu.node: cannot allocate memory in static TLS block</code></p><p>But I test on my OpenSUSE Linux(x64 gnu) with nodeV14.17.1, it runs successfully.</p><h2 id="source-code" tabindex="-1">Source Code <a class="header-anchor" href="#source-code" aria-hidden="true">#</a></h2><div class="language-shell"><pre><code><span class="token function">git</span> clone --recursive https://github.com/timzaak/spa-server
<span class="token builtin class-name">cd</span> jsclient <span class="token operator">&amp;&amp;</span> <span class="token function">yarn</span> <span class="token function">install</span> <span class="token operator">&amp;&amp;</span> <span class="token function">yarn</span> build
</code></pre></div><h3 id="command-line" tabindex="-1">Command Line <a class="header-anchor" href="#command-line" aria-hidden="true">#</a></h3><p>You can install <code>spa-client</code> commandline tool if the npm package does not support your OS platform. read <a href="./spa-client-command-line.html#source-code">doc</a>.</p>`,15),s=[r];function i(o,c,p,l,h,m){return a(),e("div",null,s)}var x=t(n,[["render",i]]);export{g as __pageData,x as default};