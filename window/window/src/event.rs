use crate::{Message, WindowId};
use libcommon::warn;
use serde::{Deserialize, Serialize};
use tao::event_loop::EventLoopProxy;

#[derive(Debug)]
pub(crate) enum UserEvent {
    IpcMessage(WindowId, String),
    SysWindowEvent(WindowId, SysWindowEvent),
    IcpResultSend(WindowId, IpcResp),
}

unsafe impl Send for UserEvent {}

#[derive(Debug, strum::Display, strum::EnumString)]
pub(crate) enum SysWindowEvent {
    DragStart,
    Close,
    Minimize,
}

impl UserEvent {
    pub(crate) fn send(self, proxy: &EventLoopProxy<UserEvent>) {
        if proxy.send_event(self).is_err() {
            warn!("Failed to send event as the event loop has been closed");
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct IpcReq {
    pub(crate) id: u32,
    pub(crate) command: String,
    pub(crate) payload: Message,
}

unsafe impl Send for IpcReq {}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct IpcResp {
    pub(crate) id: u32,
    pub(crate) payload: Option<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<String>,
}

impl IpcResp {
    pub fn ok(id: u32, payload: Message) -> Self {
        Self {
            id,
            payload: Some(payload),
            error: None,
        }
    }

    pub fn err(id: u32, error: String) -> Self {
        Self {
            id,
            payload: None,
            error: Some(error),
        }
    }
}
