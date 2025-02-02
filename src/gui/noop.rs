use tokio::{
    runtime::Handle,
    sync::mpsc::{Sender, UnboundedReceiver},
};

pub fn gui_main(
    handle: &Handle,
    close_sx: Sender<()>,
    mut gui_rx: UnboundedReceiver<super::GuiMessage>,
) {
}
