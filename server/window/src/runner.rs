use crate::{
    UserEvent, WindowConfig, WindowManager, WindowOpExt, WindowRef, utils::call_on_key_from_window,
};
use libcommon::prelude::{Result, info};
use std::{cell::RefCell, rc::Rc};
use tao::{
    event::{DeviceEvent, Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopWindowTarget},
    window::WindowBuilder,
};
use wry::WebViewBuilder;

/// 窗口执行
pub struct WindowRunner {
    el: EventLoop<UserEvent>,
    wm: Rc<RefCell<WindowManager>>,
}

impl WindowRunner {
    pub(crate) fn new(wm: WindowManager) -> Self {
        Self {
            el: EventLoopBuilder::with_user_event().build(),
            wm: Rc::new(RefCell::new(wm)),
        }
    }

    pub(crate) fn create_window_impl(
        target: &EventLoopWindowTarget<UserEvent>,
        config: &WindowConfig,
    ) -> Result<WindowRef> {
        let window = config.build_window(WindowBuilder::new()).build(target)?;
        let webview = config.build_webview(WebViewBuilder::new()).build(&window)?;
        let id = window.id();
        let label = config.title.clone();
        let w_ref = WindowRef {
            id,
            label,
            window,
            webview,
        };
        Ok(w_ref)
    }

    /// 运行窗口系统
    pub fn run(self) -> ! {
        {
            let wm = self.wm.borrow();
            let proxy = self.el.create_proxy();
            let pending = wm.pending.take();
            for wc in pending {
                let _ = proxy.send_event(UserEvent::Create(wc)); // 会缓存未运行前的event
            }
            wm.setup_proxy(proxy);
        }
        self.el.run(move |event, target, control_flow| {
            *control_flow = ControlFlow::Wait;
            // info!("event: {event:?}");
            match event {
                Event::DeviceEvent {
                    event: DeviceEvent::Key(key),
                    ..
                } => {
                    // info!("Event::DeviceEvent: {key:?}");
                    call_on_key_from_window(&key);
                }
                Event::WindowEvent {
                    event, window_id, ..
                } => match event {
                    WindowEvent::Focused(focused) => {
                        // info!("Event::WindowEvent::Focused: {window_id:?} {focused:?}");
                        {
                            self.wm.borrow().set_curr_focused(window_id, focused);
                        }
                    }
                    WindowEvent::CloseRequested => {
                        info!("Event::WindowEvent::CloseRequested: {window_id:?}");
                        let is_empty = {
                            let wm = self.wm.borrow();
                            let _ = wm.close(window_id);
                            wm.is_empty()
                        };
                        if is_empty {
                            let on_close = { self.wm.borrow().listeners.write().on_close.take() };
                            if let Some(on_close) = on_close {
                                info!("Call on_close()");
                                on_close();
                            }
                            info!("EXIT");
                            *control_flow = ControlFlow::Exit;
                        }
                    }

                    _ => {}
                },
                Event::RedrawEventsCleared => {}
                Event::MainEventsCleared => {}
                Event::NewEvents(StartCause::Init) => {
                    info!("Event::NewEvents: INIT");
                    if let Some(on_setup) = { self.wm.borrow().listeners.write().on_setup.take() } {
                        info!("Call on_setup()");
                        on_setup();
                    }
                }
                Event::UserEvent(event) => {
                    info!("Event::UserEvent: {event:?}");
                    match event {
                        UserEvent::Create(wc) => {
                            if let Ok(w_ref) = Self::create_window_impl(target, &wc) {
                                {
                                    self.wm.borrow().set_curr_focused(w_ref.id, true);
                                }
                                {
                                    self.wm.borrow().insert_created_window(w_ref);
                                }
                            }
                        }
                        UserEvent::Exit => {
                            info!("EXIT");
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                }
                _ => {}
            }
        });
    }
}
