use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::Arc,
};

use http::{Method, Response, Uri};
use http_body_util::BodyExt;
use hudsucker::{rustls::crypto::aws_lc_rs, HttpHandler};
use os::ProxyConfigs;
mod os;

type OneShotSender = tokio::sync::mpsc::Sender<()>;

#[derive(Debug, Clone)]
struct Handler {
    fetch_score: Uri,
    taiko_songsscore: Uri,

    current_uri: Option<Uri>,
    fetched_score_response: Arc<tokio::sync::Mutex<Option<String>>>,
    finished_sx: Option<OneShotSender>,
}

impl Handler {
    pub fn new(sx: OneShotSender) -> Self {
        let fetch_score: Uri = "https://www.baidu.com/api/ahfsdafbaqwerhue"
            .parse()
            .unwrap();
        let taiko_songsscore: Uri = "https://wl-taiko.wahlap.net/api/user/profile/songscore"
            .parse()
            .unwrap();
        Self {
            fetch_score,
            taiko_songsscore,
            current_uri: None,
            fetched_score_response: Default::default(),
            finished_sx: Some(sx),
        }
    }
}

impl HttpHandler for Handler {
    async fn handle_response(
        &mut self,
        _ctx: &hudsucker::HttpContext,
        res: hudsucker::hyper::Response<hudsucker::Body>,
    ) -> hudsucker::hyper::Response<hudsucker::Body> {
        if self.current_uri.as_ref() == Some(&self.taiko_songsscore) {
            tracing::info!("正在解析分数数据响应数据");
            let res = hudsucker::decode_response(res).expect("解析分数响应数据失败");

            let (parts, body) = res.into_parts();

            let body = body.collect().await.unwrap().to_bytes();

            // println!("成功捕获到分数数据 {:?}", body);
            tracing::info!("成功捕获到分数数据，大小为 {}", body.len());

            let cloned_body = http_body_util::Full::new(body.clone());

            *self.fetched_score_response.lock().await =
                Some(String::from_utf8_lossy(&body).into_owned());

            self.current_uri = None;

            return Response::from_parts(parts, cloned_body.into());
        } else if self.current_uri.as_ref() == Some(&self.fetch_score) {
            if let Some(fetched_score_response) = self.fetched_score_response.lock().await.as_ref()
            {
                tracing::info!("监测到同步接口请求，正在转发捕获到的分数数据");
                let fetched_score_response = fetched_score_response.clone();

                if let Some(sx) = self.finished_sx.take() {
                    tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                        sx.send(()).await.unwrap();
                    });
                }

                return hudsucker::hyper::Response::builder()
                    .header("Content-Type", "applicaiton/json")
                    .header("X-Data-Fetched", "1")
                    .status(200)
                    .version(res.version())
                    .body(fetched_score_response.into())
                    .unwrap();
            } else {
                tracing::warn!("监测到同步接口请求，但是并没有获取到任何分数数据，请先从鼓众广场小程序中点击我的分数查询！");
            };
        }

        self.current_uri = None;
        res
    }

    async fn handle_request(
        &mut self,
        _ctx: &hudsucker::HttpContext,
        req: http::Request<hudsucker::Body>,
    ) -> hudsucker::RequestOrResponse {
        self.current_uri = Some(req.uri().clone());

        if req.uri() == &self.taiko_songsscore && req.method() == Method::POST {
            tracing::debug!("检测到分数接口请求");
        } else if req.uri() == &self.fetch_score && req.method() == Method::GET {
            tracing::debug!("检测到成绩同步接口请求");
        }

        req.into()
    }
}

pub fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .expect("无法获取配置目录")
        .join("taiko-score-getter")
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let listen_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 7650);
    let (sx, mut rx) = tokio::sync::mpsc::channel(1);

    let proxy_configs = ProxyConfigs::new().await;
    proxy_configs
        .set_proxy(listen_addr.ip().to_string(), listen_addr.port())
        .await;

    tracing::info!("正在启动代理服务器 {}", listen_addr);

    let proxy = hudsucker::Proxy::builder()
        .with_addr(listen_addr)
        .with_ca(os::get_ca().await)
        .with_rustls_client(aws_lc_rs::default_provider())
        .with_http_handler(Handler::new(sx))
        .with_graceful_shutdown(async move {
            tracing::info!("代理服务器已启动！");

            tokio::select! {
                v = tokio::signal::ctrl_c() => {
                    v.expect("Failed to install CTRL+C signal handler");
                },
                _ = rx.recv() => {
                    tracing::info!("已完成成绩同步，准备关闭程序");
                }
            };

            tracing::info!("正在关闭代理服务器！");
        })
        .build()
        .unwrap();

    proxy.start().await.unwrap();

    tracing::info!("正在还原代理配置");
    proxy_configs.recover().await;
}
