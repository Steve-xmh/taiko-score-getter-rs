use cacao::{
    appkit::{
        menu::{Menu, MenuItem},
        window::{Window, WindowConfig, WindowDelegate, WindowStyle},
        App, AppDelegate,
    },
    color::Color,
    layout::{Layout, LayoutConstraint},
    notification_center::Dispatcher,
    text::Label,
    view::View,
};
use objc2::{AllocAnyThread, MainThreadMarker};
use objc2_app_kit::{NSApplication, NSImage};
use tokio::{
    runtime::Handle,
    sync::mpsc::{Sender, UnboundedReceiver},
};

use crate::gui::GuiMessage;

#[derive(Debug)]
struct StatusApp {
    window: Window<MainWindowDelegate>,
}

#[derive(Debug)]
struct MainWindowDelegate {
    close_sx: Sender<()>,

    content: View,

    label_launch_proxy: Label,
    label_receive_score: Label,
    label_sync_score: Label,

    label_description: Label,
}

impl MainWindowDelegate {
    fn layout(&self) {
        self.label_launch_proxy.set_text("1. 初始化代理服务器");
        self.label_receive_score.set_text("2. 等待接收分数数据");
        self.label_sync_score.set_text("3. 等待同步分数操作");

        self.label_launch_proxy.set_text_color(Color::SystemBlue);

        self.label_description
            .set_text("正在初始化证书和代理服务器……");

        // set width to fill parent view
    }
}

impl WindowDelegate for MainWindowDelegate {
    const NAME: &'static str = "MainWindowDelegate";

    fn did_load(&mut self, window: Window) {
        window.set_content_view(&self.content);

        self.content.add_subview(&self.label_launch_proxy);
        self.content.add_subview(&self.label_receive_score);
        self.content.add_subview(&self.label_sync_score);
        self.content.add_subview(&self.label_description);

        LayoutConstraint::activate(&[
            self.content.width.constraint_equal_to_constant(350.0),
            self.label_launch_proxy
                .top
                .constraint_equal_to(&self.content.top)
                .offset(0.0),
            self.label_launch_proxy
                .leading
                .constraint_equal_to(&self.content.leading)
                .offset(10.0),
            self.label_launch_proxy
                .trailing
                .constraint_equal_to(&self.content.trailing)
                .offset(-10.0),
            self.label_receive_score
                .top
                .constraint_equal_to(&self.label_launch_proxy.bottom)
                .offset(10.0),
            self.label_receive_score
                .leading
                .constraint_equal_to(&self.content.leading)
                .offset(10.0),
            self.label_receive_score
                .trailing
                .constraint_equal_to(&self.content.trailing)
                .offset(-10.0),
            self.label_sync_score
                .top
                .constraint_equal_to(&self.label_receive_score.bottom)
                .offset(10.0),
            self.label_sync_score
                .leading
                .constraint_equal_to(&self.content.leading)
                .offset(10.0),
            self.label_sync_score
                .trailing
                .constraint_equal_to(&self.content.trailing)
                .offset(-10.0),
            self.label_description
                .top
                .constraint_equal_to(&self.label_sync_score.bottom)
                .offset(10.0),
            self.label_description
                .leading
                .constraint_equal_to(&self.content.leading)
                .offset(10.0),
            self.label_description
                .trailing
                .constraint_equal_to(&self.content.trailing)
                .offset(-10.0),
            self.label_description
                .bottom
                .constraint_equal_to(&self.content.bottom)
                .offset(-10.0),
        ]);
    }

    fn will_close(&self) {
        let _ = self.close_sx.blocking_send(());
    }
}

impl AppDelegate for StatusApp {
    fn did_finish_launching(&self) {
        App::set_menu(vec![Menu::new("Taiko Score Getter", vec![MenuItem::Quit])]);

        App::activate();

        let icon_data = include_bytes!("../../assets/icon.png");
        let icon_data = objc2_foundation::NSData::with_bytes(icon_data);
        let logo = NSImage::initWithData(NSImage::alloc(), &icon_data).expect("无法加载图片");

        unsafe {
            let nsapp = NSApplication::sharedApplication(MainThreadMarker::new().unwrap());
            nsapp.setApplicationIconImage(Some(&logo));
        }

        self.window.set_titlebar_appears_transparent(true);
        self.window.show();
        self.window.set_title("Taiko Score Getter 太鼓成绩获取工具");
        self.window.delegate.as_ref().unwrap().layout();
    }

    fn should_terminate_after_last_window_closed(&self) -> bool {
        true
    }
}

impl Dispatcher for StatusApp {
    type Message = GuiMessage;

    fn on_ui_message(&self, msg: Self::Message) {
        let delegate = self.window.delegate.as_ref().unwrap();
        match msg {
            GuiMessage::Init => {}
            GuiMessage::WaitForScoreData => {
                delegate
                    .label_launch_proxy
                    .set_text_color(Color::SystemGreen);
                delegate
                    .label_receive_score
                    .set_text_color(Color::SystemBlue);

                delegate
                    .label_description
                    .set_text("请打开 鼓众广场 小程序，点击 游戏成绩 按钮，等待程序接收成绩信息。");
            }
            GuiMessage::WaitForScoreSync => {
                delegate
                    .label_receive_score
                    .set_text_color(Color::SystemGreen);
                delegate.label_sync_score.set_text_color(Color::SystemBlue);

                delegate.label_description.set_text("最后，请打开 Don Note 小程序，切换到 数据同步 页面，点击 成绩同步 按钮，即可完成数据同步啦！");
            }
            GuiMessage::SendingScoreData => {
                delegate.label_sync_score.set_text_color(Color::SystemGreen);

                delegate
                    .label_description
                    .set_text("成绩数据已同步完成！程序即将在 3 秒后退出……");
            }
            GuiMessage::Close => {
                self.window.close();
            }
        }
    }
}

pub fn gui_main(
    handle: &Handle,
    close_sx: Sender<()>,
    mut gui_rx: UnboundedReceiver<super::GuiMessage>,
) {
    tracing::info!("GUI 线程已启动");

    handle.spawn(async move {
        while let Some(msg) = gui_rx.recv().await {
            cacao::appkit::App::<StatusApp, GuiMessage>::dispatch_main(msg);
        }
    });

    let mut win_cfg = WindowConfig::default();

    win_cfg.set_styles(&[WindowStyle::Closable, WindowStyle::Titled]);

    App::new(
        "net.stevexmh.taikoscoregetter",
        StatusApp {
            window: Window::with(
                win_cfg,
                MainWindowDelegate {
                    close_sx,
                    label_launch_proxy: Label::default(),
                    label_receive_score: Label::default(),
                    label_sync_score: Label::default(),
                    label_description: Label::default(),
                    content: View::default(),
                },
            ),
        },
    )
    .run();
}
