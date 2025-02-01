use std::sync::Mutex;

use tokio::sync::mpsc::{Sender, UnboundedSender};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows::*;

#[cfg(not(target_os = "windows"))]
mod noop;
#[cfg(not(target_os = "windows"))]
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

pub fn init_gui(close_sx: Sender<()>) {
    let (gui_sx, gui_rx) = tokio::sync::mpsc::unbounded_channel::<GuiMessage>();

    tokio::task::spawn_blocking(move || gui_main(close_sx, gui_rx));

    GUI_SX.lock().unwrap().replace(gui_sx);
}

pub fn send_msg_to_gui(msg: GuiMessage) {
    if let Some(sx) = GUI_SX.lock().unwrap().as_ref() {
        let _ = sx.send(msg);
    }
}
