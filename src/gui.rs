use std::sync::Mutex;

use tokio::{
    runtime::Handle,
    sync::mpsc::{Sender, UnboundedSender},
};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos::*;

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod noop;
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
use noop::*;

#[derive(Debug, Clone)]
pub enum GuiMessage {
    Init,
    // InitError(String),
    WaitForScoreData,
    WaitForScoreSync,
    SendingScoreData,
    Close,
}

static GUI_SX: Mutex<Option<UnboundedSender<GuiMessage>>> = Mutex::new(None);

pub fn init_gui(handle: &Handle, close_sx: Sender<()>) {
    let (gui_sx, gui_rx) = tokio::sync::mpsc::unbounded_channel::<GuiMessage>();

    GUI_SX.lock().unwrap().replace(gui_sx.clone());
    gui_main(handle, close_sx, gui_rx);
}

pub fn send_msg_to_gui(msg: GuiMessage) {
    if let Some(sx) = GUI_SX.lock().unwrap().as_ref() {
        let _ = sx.send(msg);
    }
}
