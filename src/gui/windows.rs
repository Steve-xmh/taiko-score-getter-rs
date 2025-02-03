use std::sync::{Arc, Mutex};

use tokio::{
    runtime::Handle,
    sync::mpsc::{Sender, UnboundedReceiver},
};
use winsafe::{
    co::{BKMODE, WS_EX},
    gui::*,
    prelude::*,
    AnyResult, COLORREF, HBRUSH,
};

const LABEL_PROGRESS_COLOR: COLORREF = COLORREF::new(0x24, 0x95, 0xFF);
const LABEL_SUCCESS_COLOR: COLORREF = COLORREF::new(0x38, 0xD0, 0x6B);
// const LABEL_ERROR_COLOR: COLORREF = COLORREF::new(0xFF, 0x00, 0x00);
const LABEL_DEFAULT_COLOR: COLORREF = COLORREF::new(0x00, 0x00, 0x00);

#[derive(Default, Debug, Clone, Copy)]
enum LabelColor {
    #[default]
    Default,
    Progress,
    Success,
    // Error,
}

impl LabelColor {
    fn as_colorref(&self) -> COLORREF {
        match self {
            LabelColor::Progress => LABEL_PROGRESS_COLOR,
            LabelColor::Success => LABEL_SUCCESS_COLOR,
            // LabelColor::Error => LABEL_ERROR_COLOR,
            _ => LABEL_DEFAULT_COLOR,
        }
    }
}

type LabelColorRef = Arc<Mutex<LabelColor>>;

pub fn gui_main(
    handle: &Handle,
    close_sx: Sender<()>,
    mut gui_rx: UnboundedReceiver<super::GuiMessage>,
) {
    tracing::info!("正在初始化 GUI 窗口……");
    let win = WindowMain::new(WindowMainOpts {
        class_name: "TaikoScoreGetter".to_string(),
        title: "Taiko Score Getter 太鼓成绩获取工具".to_string(),
        size: (560, 115),
        class_icon: Icon::Id(1),
        class_bg_brush: Brush::Handle(
            HBRUSH::CreateSolidBrush(COLORREF::new(0xFF, 0xFF, 0xFF))
                .unwrap()
                .leak(),
        ),
        ex_style: WS_EX::LEFT | WS_EX::TOPMOST,
        ..Default::default()
    });

    let label_launch_proxy_color = LabelColorRef::default();
    let label_launch_proxy = Label::new(
        &win,
        LabelOpts {
            text: "1. 初始化代理服务器".to_string(),
            position: (10, 10),
            size: (560 - 10 - 10, 20),
            ..Default::default()
        },
    );

    let label_receive_score_color = LabelColorRef::default();
    let label_receive_score = Label::new(
        &win,
        LabelOpts {
            text: "2. 等待接收分数数据".to_string(),
            position: (10, 10 + 20),
            size: (560 - 10 - 10, 20),
            ..Default::default()
        },
    );

    let label_sync_score_color = LabelColorRef::default();
    let label_sync_score = Label::new(
        &win,
        LabelOpts {
            text: "3. 等待同步分数操作".to_string(),
            position: (10, 10 + 20 * 2),
            size: (560 - 10 - 10, 20),
            ..Default::default()
        },
    );

    let label_description = Label::new(
        &win,
        LabelOpts {
            text: "正在初始化证书和代理服务器……".to_string(),
            position: (10, 10 + 20 * 4),
            size: (560 - 10 - 10, 50),
            ..Default::default()
        },
    );

    win.on().wm_close({
        let win = win.clone();
        let close_sx = close_sx.clone();
        move || {
            let _ = close_sx.try_send(());
            let _ = win.hwnd().DestroyWindow();
            Ok(())
        }
    });

    win.on().wm_ctl_color_static({
        let label_launch_proxy = label_launch_proxy.clone();
        let label_sync_score = label_sync_score.clone();
        let label_receive_score = label_receive_score.clone();

        let label_launch_proxy_color = label_launch_proxy_color.clone();
        let label_receive_score_color = label_receive_score_color.clone();
        let label_sync_score_color = label_sync_score_color.clone();

        move |params| {
            let to_brush = |color: COLORREF| -> AnyResult<HBRUSH> {
                let brush = HBRUSH::CreateSolidBrush(color)?;
                let mut brush = params.hdc.SelectObject(&*brush)?;
                params.hdc.SetTextColor(color)?;
                params.hdc.SetBkMode(BKMODE::TRANSPARENT)?;
                Ok(brush.leak())
            };

            if label_launch_proxy.hwnd() == &params.hwnd {
                let target_color = label_launch_proxy_color
                    .lock()
                    .expect("无法获取标签 1 颜色")
                    .as_colorref();

                to_brush(target_color)
            } else if label_receive_score.hwnd() == &params.hwnd {
                let target_color = label_receive_score_color
                    .lock()
                    .expect("无法获取标签 2 颜色")
                    .as_colorref();

                to_brush(target_color)
            } else if label_sync_score.hwnd() == &params.hwnd {
                let target_color = label_sync_score_color
                    .lock()
                    .expect("无法获取标签 3 颜色")
                    .as_colorref();

                to_brush(target_color)
            } else {
                let brush = HBRUSH::CreateSolidBrush(COLORREF::default())?;
                let mut brush = params.hdc.SelectObject(&*brush)?;
                params.hdc.SetTextColor(LABEL_DEFAULT_COLOR)?;
                params.hdc.SetBkColor(COLORREF::new(0xFF, 0xFF, 0xFF))?;
                params.hdc.SetBkMode(BKMODE::TRANSPARENT)?;
                Ok(brush.leak())
            }
        }
    });

    tokio::spawn({
        let win = win.clone();

        let label_launch_proxy = label_launch_proxy.clone();
        let label_sync_score = label_sync_score.clone();
        let label_receive_score = label_receive_score.clone();
        let label_description = label_description.clone();

        let label_launch_proxy_color = label_launch_proxy_color.clone();
        let label_receive_score_color = label_receive_score_color.clone();
        let label_sync_score_color = label_sync_score_color.clone();

        async move {
            while let Some(evt) = gui_rx.recv().await {
                match evt {
                    super::GuiMessage::Init => {
                        *label_launch_proxy_color.lock().unwrap() = LabelColor::Progress;

                        win.run_ui_thread({
                            let win = win.clone();

                            let label_launch_proxy = label_launch_proxy.clone();
                            let label_sync_score = label_sync_score.clone();
                            let label_receive_score = label_receive_score.clone();
                            let label_description = label_description.clone();

                            move || {
                                win.hwnd().InvalidateRect(None, true)?;

                                win.hwnd().InvalidateRect(None, true)?;
                                label_launch_proxy.hwnd().InvalidateRect(None, true)?;
                                label_sync_score.hwnd().InvalidateRect(None, true)?;
                                label_receive_score.hwnd().InvalidateRect(None, true)?;
                                label_description.hwnd().InvalidateRect(None, true)?;

                                Ok(())
                            }
                        });
                    }
                    super::GuiMessage::CertTrustNeeded => {
                        // 一般不会出现需要验证证书信任的这个情况
                    }
                    // super::GuiMessage::InitError(_msg) => {
                    //     // TODO: 显示点什么？
                    // }
                    super::GuiMessage::WaitForScoreData => {
                        *label_launch_proxy_color.lock().unwrap() = LabelColor::Success;
                        *label_receive_score_color.lock().unwrap() = LabelColor::Progress;

                        win.run_ui_thread({
                            let win = win.clone();

                            let label_launch_proxy = label_launch_proxy.clone();
                            let label_sync_score = label_sync_score.clone();
                            let label_receive_score = label_receive_score.clone();
                            let label_description = label_description.clone();
                            move || {
                                label_description
                                    .set_text_and_resize("请打开 鼓众广场 小程序，点击 游戏成绩 按钮，等待程序接收成绩信息。");

                                    win.hwnd().InvalidateRect(None, true)?;
                                    label_launch_proxy.hwnd().InvalidateRect(None, true)?;
                                    label_sync_score.hwnd().InvalidateRect(None, true)?;
                                    label_receive_score.hwnd().InvalidateRect(None, true)?;
                                    label_description.hwnd().InvalidateRect(None, true)?;

                                Ok(())
                            }
                        });
                    }
                    super::GuiMessage::WaitForScoreSync => {
                        *label_receive_score_color.lock().unwrap() = LabelColor::Success;
                        *label_sync_score_color.lock().unwrap() = LabelColor::Progress;

                        win.run_ui_thread({
                            let win = win.clone();

                            let label_launch_proxy = label_launch_proxy.clone();
                            let label_sync_score = label_sync_score.clone();
                            let label_receive_score = label_receive_score.clone();
                            let label_description = label_description.clone();
                            move || {
                                label_description
                                    .set_text_and_resize("最后，请打开 Don Note 小程序，切换到 数据同步 页面，点击 成绩同步 按钮，即可完成数据同步啦！");

                                win.hwnd().InvalidateRect(None, true)?;
                                label_launch_proxy.hwnd().InvalidateRect(None, true)?;
                                label_sync_score.hwnd().InvalidateRect(None, true)?;
                                label_receive_score.hwnd().InvalidateRect(None, true)?;
                                label_description.hwnd().InvalidateRect(None, true)?;

                                Ok(())
                            }
                        });
                    }
                    super::GuiMessage::SendingScoreData => {
                        *label_sync_score_color.lock().unwrap() = LabelColor::Success;

                        win.run_ui_thread({
                            let win = win.clone();

                            let label_launch_proxy = label_launch_proxy.clone();
                            let label_sync_score = label_sync_score.clone();
                            let label_receive_score = label_receive_score.clone();
                            let label_description = label_description.clone();

                            move || {
                                label_description.set_text_and_resize(
                                    "成绩数据已同步完成！程序即将在 3 秒后退出……",
                                );

                                win.hwnd().InvalidateRect(None, true)?;
                                label_launch_proxy.hwnd().InvalidateRect(None, true)?;
                                label_sync_score.hwnd().InvalidateRect(None, true)?;
                                label_receive_score.hwnd().InvalidateRect(None, true)?;
                                label_description.hwnd().InvalidateRect(None, true)?;

                                Ok(())
                            }
                        });
                    }
                    super::GuiMessage::Close => {
                        tracing::info!("正在关闭 GUI……");

                        win.run_ui_thread({
                            let win = win.clone();

                            move || {
                                win.hwnd().DestroyWindow()?;
                                Ok(())
                            }
                        });
                        return;
                    }
                }
            }
        }
    });

    win.run_main(None).unwrap();
}
