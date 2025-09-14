use crate::{App, UserEvent, WindowConfig, WindowHandle};
use libcommon::prelude::{Result, info};
use std::{cell::RefCell, rc::Rc};
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopWindowTarget},
};
use wry::WebViewBuilder;

/// 窗口执行
pub(crate) struct WindowRunner {
    el: EventLoop<UserEvent>,
    wm: Rc<RefCell<App>>,
}
impl WindowRunner {
    pub(crate) fn with_manager(wm: App) -> Self {
        Self {
            el: EventLoopBuilder::with_user_event().build(),
            wm: Rc::new(RefCell::new(wm)),
        }
    }

    pub(crate) fn create_window_impl(
        target: &EventLoopWindowTarget<UserEvent>,
        wc: &WindowConfig,
    ) -> Result<WindowHandle> {
        let window = wc
            .build_window(tao::window::WindowBuilder::new())
            .build(target)?;
        let webview = wc.build_webview(WebViewBuilder::new()).build(&window)?;
        let id = window.id();
        Ok(WindowHandle {
            id,
            label: wc.label.clone(),
            _window: window,
            _webview: webview,
        })
    }

    pub(crate) fn run(self) -> ! {
        {
            let wm = self.wm.borrow();
            let proxy = self.el.create_proxy();

            let pending_windows = std::mem::take(&mut wm.pending.take());
            for wc in pending_windows {
                let _ = proxy.send_event(UserEvent::Create(wc)); // 会缓存未运行前的event
            }
            wm.setup_proxy(proxy);
        }
        self.el.run(move |event, target, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    let wm = self.wm.borrow();
                    let _ = wm.close_impl(window_id);
                    info!("Event: CloseRequested: {window_id:?}");
                    if wm.empty() {
                        info!("Event: Exit");
                        *control_flow = ControlFlow::Exit
                    }
                }
                Event::MainEventsCleared => {}
                Event::RedrawRequested(_) => {}
                Event::UserEvent(e) => {
                    info!("Event: {e}");
                    match e {
                        UserEvent::Create(wc) => {
                            if let Ok(wh) = Self::create_window_impl(target, &wc) {
                                let _ = self.wm.borrow_mut().add_wh(wh);
                            };
                        }
                    }
                }
                _ => (),
            }
        })
    }
}
