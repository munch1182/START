use crate::{
    FnResult, RawMessage, WindowId, WindowRef, WindowState,
    event::{IpcReq, IpcResp, SysWindowEvent, UserEvent},
};
use dashmap::DashMap;
use libcommon::prelude::*;
use std::{pin::Pin, sync::Arc};
use tao::{
    event::Event,
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    window::WindowBuilder,
};
use wry::WebViewBuilder;

/// 内部存储动态分发的类型
type BoxedHandler<H> = Box<
    dyn Fn(Option<RawMessage>, WindowState<H>) -> Pin<Box<dyn Future<Output = FnResult> + Send>>
        + Send
        + Sync,
>;

pub struct WindowManager<H> {
    wm: DashMap<WindowId, WindowRef>,
    event: EventLoop<UserEvent>,
    handlers: Arc<DashMap<String, Arc<BoxedHandler<H>>>>,
    state: WindowState<H>,
}

impl Default for WindowManager<()> {
    fn default() -> Self {
        Self {
            wm: Default::default(),
            event: EventLoopBuilder::with_user_event().build(),
            handlers: Default::default(),
            state: WindowState(().into()),
        }
    }
}

impl<H: Send + Sync + 'static> WindowManager<H> {
    pub fn with_state(state: Arc<H>) -> Self {
        Self {
            wm: DashMap::new(),
            event: EventLoopBuilder::with_user_event().build(),
            handlers: Arc::new(DashMap::new()),
            state: WindowState(state),
        }
    }

    pub fn create<'a>(&self, win: WindowBuilder, web: WebViewBuilder<'a>) -> Result<WindowId> {
        let wref = WindowRef::create(win, web, &self.event)?;
        let id = wref.id();
        self.wm.insert(id.clone(), wref);
        Ok(id)
    }

    pub fn run(self) -> ! {
        let proxy = self.event.create_proxy();
        self.event.run(move |event, _, flow| {
            *flow = ControlFlow::Wait;
            match event {
                Event::Opened { urls } => {
                    debug!("open: {:?}", urls);
                }
                Event::UserEvent(user_event) => match user_event {
                    UserEvent::IpcMessage(wid, msg) => {
                        let ipcreq: IpcReq = match serde_json::from_str(&msg) {
                            Ok(req) => req,
                            Err(e) => return warn!("Failed to parse ipc message: {e}, ignore."),
                        };
                        trace!("receiver IpcMessage: {ipcreq:?}");
                        let cmd = ipcreq.command;
                        let Some(fun) = self.handlers.get(&cmd).map(|v| Arc::clone(&v)) else {
                            let resp = IpcResp::err(
                                ipcreq.id,
                                format!("No handler registered for command '{cmd}'"),
                            );
                            UserEvent::IcpResultSend(wid, resp).send(&proxy);
                            return;
                        };
                        let proxy = proxy.clone();
                        let state = self.state.clone();
                        tokio::spawn(async move {
                            let resp = match fun(ipcreq.payload, state).await {
                                Ok(res) => IpcResp::ok(ipcreq.id, res),
                                Err(e) => IpcResp::err(ipcreq.id, format!("{e:?}")),
                            };
                            trace!("resp to: {wid}: {resp:?}");
                            UserEvent::IcpResultSend(wid, resp).send(&proxy);
                        });
                    }
                    UserEvent::IcpResultSend(wid, resp) => {
                        if let Some(w) = self.wm.get(&wid)
                            && let Err(e) = w.resp2web(resp)
                        {
                            warn!("Failed to send ipc result to window({wid}): {e}");
                        }
                    }
                    UserEvent::SysWindowEvent(id, sys) => match sys {
                        SysWindowEvent::DragStart => {
                            if let Some(w) = self.wm.get(&id)
                                && let Err(e) = w.window.drag_window()
                            {
                                warn!("Failed to drag window({id}): {e}");
                            }
                        }
                        SysWindowEvent::Close => {
                            if let Some(w) = self.wm.remove(&id) {
                                drop(w);
                                debug!("Window({id}) closed");
                            }
                            if self.wm.is_empty() {
                                info!("All windows closed, exiting");
                                *flow = ControlFlow::Exit;
                            }
                        }
                        SysWindowEvent::Minimize => {
                            if let Some(r) = self.wm.get(&id) {
                                r.window.set_minimized(true);
                            }
                        }
                    },
                },
                _ => {}
            }
        })
    }

    pub fn register_handler<I, F>(&self, handlers: I)
    where
        I: IntoIterator<Item = (String, F)>,
        F: Fn(
                Option<RawMessage>,
                WindowState<H>,
            ) -> Pin<Box<dyn Future<Output = FnResult> + Send + 'static>>
            + Send
            + Sync
            + 'static,
    {
        for (name, ele) in handlers {
            self.handlers.insert(name, Arc::new(Box::new(ele)));
        }
    }
}

pub trait WindowCreateExt<T, E> {
    fn create_window(&self, win: T, web: E) -> Result<WindowId>;
}

impl<T: ToString, E: ToString, H: Send + Sync + 'static> WindowCreateExt<T, E>
    for WindowManager<H>
{
    fn create_window(&self, title: T, url: E) -> Result<WindowId> {
        let win = WindowBuilder::new()
            .with_title(title.to_string())
            .with_decorations(false);
        let web = WebViewBuilder::new().with_url(url.to_string());
        self.create(win, web)
    }
}
