use crate::{
    event::{IpcResp, SysWindowEvent, UserEvent},
    script,
};
use libcommon::{New, hash, prelude::*};
use std::sync::Arc;
use tao::{
    event_loop::{EventLoop, EventLoopProxy},
    window::{Window as TaoWindow, WindowBuilder},
};
use wry::{WebView as WryWebView, WebViewBuilder, http::Request};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct WindowId(pub Arc<String>);

pub struct WindowState<H>(pub Arc<H>);

impl<H> Clone for WindowState<H> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[derive(New)]
pub(crate) struct WindowRef {
    pub(crate) window: TaoWindow,
    pub(crate) webview: WryWebView,
}

impl WindowRef {
    pub(crate) fn resp2web(&self, resp: IpcResp) -> Result<()> {
        let json = serde_json::to_string(&resp)?;
        self.webview
            .evaluate_script(&script::bridge_handler_call(&json))?;
        Ok(())
    }

    pub(crate) fn id(&self) -> WindowId {
        (&self.window).into()
    }

    pub(crate) fn create<'a>(
        win: WindowBuilder,
        web: WebViewBuilder<'a>,
        event: &EventLoop<UserEvent>,
    ) -> Result<Self> {
        let window = win.build(event)?;
        let wid: WindowId = (&window).into();
        let proxy = event.create_proxy();
        let webview = web
            .with_initialization_script(script::setup_script())
            .with_ipc_handler(move |req| Self::handle_ipc(&proxy, &wid, req))
            .build(&window)?;
        Ok(Self::new(window, webview))
    }

    pub(crate) fn handle_ipc(
        proxy: &EventLoopProxy<UserEvent>,
        wid: &WindowId,
        req: Request<String>,
    ) {
        let cmd = req.body().to_string();
        debug!("Received IPC message: {cmd}");
        if let Ok(sys) = cmd.parse::<SysWindowEvent>() {
            UserEvent::SysWindowEvent(wid.clone(), sys).send(proxy);
        } else {
            UserEvent::IpcMessage(wid.clone(), cmd).send(proxy);
        }
    }
}

impl<H> WindowState<H> {
    pub fn get(&self) -> &H {
        &self.0
    }
}

impl std::fmt::Display for WindowId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&TaoWindow> for WindowId {
    fn from(value: &TaoWindow) -> Self {
        Self(hash!(&value.id()).to_string().into())
    }
}
