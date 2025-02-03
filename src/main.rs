#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use gui::{send_msg_to_gui, GuiMessage};
use http::{Method, Response, Uri};
use http_body_util::BodyExt;
use hudsucker::HttpHandler;
use os::ProxyConfigs;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod gui;
mod os;
mod songs_score;

type OneShotSender = tokio::sync::mpsc::Sender<()>;

#[derive(Debug, Clone, Copy)]
enum UriType {
    FetchScore,
    TaikoSongScore,
}

#[derive(Debug, Clone)]
struct Handler {
    fetch_score: Uri,
    taiko_songsscore: Uri,

    current_uri_type: Option<UriType>,
    fetched_score_response: Arc<tokio::sync::Mutex<Option<String>>>,
    finished_sx: Option<OneShotSender>,
}

// https://www.baidu.com:443/api/ahfsdafbaqwerhue
// https://wl-taiko.wahlap.net:443/api/user/profile/songscore

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
            current_uri_type: None,
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
        match self.current_uri_type {
            Some(UriType::FetchScore) => {
                if let Some(fetched_score_response) =
                    self.fetched_score_response.lock().await.as_ref()
                {
                    tracing::info!("监测到同步接口请求，正在转发捕获到的分数数据");
                    let fetched_score_response = fetched_score_response.clone();

                    if let Some(sx) = self.finished_sx.take() {
                        tokio::spawn(async move {
                            tokio::time::sleep(Duration::from_secs(3)).await;
                            sx.send(()).await.unwrap();
                        });
                    }

                    send_msg_to_gui(GuiMessage::SendingScoreData);

                    return hudsucker::hyper::Response::builder()
                        .header("Content-Type", "application/json")
                        .header("X-Data-Fetched", "1")
                        .status(200)
                        .version(res.version())
                        .body(hudsucker::Body::from(fetched_score_response))
                        .unwrap();
                } else {
                    tracing::warn!("监测到同步接口请求，但是并没有获取到任何分数数据，请先从鼓众广场小程序中点击我的分数查询！");
                };
            }
            Some(UriType::TaikoSongScore) => {
                tracing::info!("正在解析分数数据响应数据");
                let res = hudsucker::decode_response(res).expect("解析分数响应数据失败");

                let (parts, body) = res.into_parts();

                let body = body.collect().await.unwrap().to_bytes();

                // println!("成功捕获到分数数据 {:?}", body);
                tracing::info!("成功捕获到分数数据，大小为 {}", body.len());

                let cloned_body = http_body_util::Full::new(body.clone());

                // let data_body = String::from_utf8_lossy(&body).into_owned();

                // tokio::fs::write("score.json", &data_body).await.unwrap();

                match serde_json::from_slice::<songs_score::Response>(&body) {
                    Ok(score_data) => {
                        tracing::info!("分数响应数据解析成功，正在生成需要返回的数据");

                        if score_data.status == 0 {
                            let mut result = Vec::with_capacity(score_data.data.score_info.len());

                            for item in score_data.data.score_info {
                                result.push(serde_json::Value::Array(Vec::from([
                                    item.song_no.into(),
                                    item.level.into(),
                                    item.high_score.into(),
                                    item.best_score_rank.into(),
                                    item.good_cnt.into(),
                                    item.ok_cnt.into(),
                                    item.ng_cnt.into(),
                                    item.pound_cnt.into(),
                                    item.combo_cnt.into(),
                                    item.stage_cnt.into(),
                                    item.clear_cnt.into(),
                                    item.full_combo_cnt.into(),
                                    item.dondaful_combo_cnt.into(),
                                    item.update_datetime.into(),
                                ])));
                            }

                            self.fetched_score_response
                                .lock()
                                .await
                                .replace(serde_json::to_string(&result).unwrap());

                            send_msg_to_gui(GuiMessage::WaitForScoreSync);
                        } else {
                            tracing::warn!("分数数据返回状态码不为 0，可能是未登录或者其他错误，响应的错误信息为：{}", score_data.message);
                        }
                    }
                    Err(err) => {
                        tracing::error!("解析分数数据失败：{}", err);
                    }
                }

                self.current_uri_type = None;

                return Response::from_parts(parts, cloned_body.into());
            }
            None => {
                return res;
            }
        }

        self.current_uri_type = None;
        res
    }

    async fn handle_request(
        &mut self,
        _ctx: &hudsucker::HttpContext,
        req: http::Request<hudsucker::Body>,
    ) -> hudsucker::RequestOrResponse {
        if req.uri().host() == self.taiko_songsscore.host() && req.method() == Method::POST {
            tracing::debug!("检测到分数接口请求: {}", req.uri());
            self.current_uri_type = Some(UriType::TaikoSongScore);
        } else if req.uri().host() == self.fetch_score.host() && req.method() == Method::GET {
            tracing::debug!("检测到成绩同步接口请求");
            self.current_uri_type = Some(UriType::FetchScore);
        } else {
            self.current_uri_type = None;
        }

        req.into()
    }
}

pub fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .expect("无法获取配置目录")
        .join("taiko-score-getter")
}

async fn proxy_main(sx: Sender<()>, mut rx: Receiver<()>) {
    let listen_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 7650);

    send_msg_to_gui(GuiMessage::Init);

    let proxy_configs = ProxyConfigs::new().await;
    proxy_configs
        .set_proxy(listen_addr.ip().to_string(), listen_addr.port())
        .await;

    tracing::info!("正在启动代理服务器 {}", listen_addr);

    let proxy = hudsucker::Proxy::builder()
        .with_addr(listen_addr)
        .with_ca(os::get_ca().await)
        .with_rustls_client(rustls::crypto::ring::default_provider())
        .with_http_handler(Handler::new(sx))
        .with_graceful_shutdown(async move {
            send_msg_to_gui(GuiMessage::WaitForScoreData);
            tracing::info!("代理服务器已启动！");

            tokio::select! {
                v = tokio::signal::ctrl_c() => {
                    v.expect("Failed to install CTRL+C signal handler");
                },
                _ = rx.recv() => {
                    tracing::info!("接收到关闭请求，准备关闭程序");
                }
            };

            tracing::info!("正在关闭代理服务器！");
        })
        .build()
        .unwrap();

    proxy.start().await.unwrap();

    tracing::info!("正在还原代理配置");
    proxy_configs.recover().await;
    tracing::info!("代理配置已还原");

    send_msg_to_gui(GuiMessage::Close);

    #[cfg(target_os = "macos")]
    {
        cacao::appkit::App::terminate();
    }
}

fn main() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .without_time()
        .compact();
    let filter =
        tracing_subscriber::filter::filter_fn(|x| x.target().starts_with("taiko_score_getter"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .build()
        .expect("无法创建异步运行时环境");

    let _guard = rt.enter();
    let (sx, rx) = tokio::sync::mpsc::channel(1);
    let task = rt.spawn(proxy_main(sx.clone(), rx));

    gui::init_gui(rt.handle(), sx);
    rt.block_on(task).unwrap();
}
