mod config;
mod event;
mod runner;
mod utils;

use crate::runner::WindowRunner;
pub use config::*;
pub use event::*;
pub use key::*;
use libcommon::prelude::*;
use parking_lot::{Mutex, RwLock};
use std::sync::Arc;
use std::{cell::RefCell, collections::HashMap};
pub use tao::platform::windows::WindowBuilderExtWindows as TaoWindowExt;
pub use tao::window::Window as TaoWindow;
pub use tao::window::WindowBuilder as TaoWindowBuilder;
use tao::{event_loop::EventLoopProxy, window::WindowId};
pub use wry::WebView as WryWebView;
pub use wry::WebViewBuilder as WryWebViewBuilder;

#[derive(Clone, Default)]
pub struct WindowManager {
    /// 持有的窗口对象
    pub(crate) wms: Arc<RwLock<HashMap<WindowId, WindowRef>>>,
    /// 窗口label和id的映射
    pub(crate) ids: Arc<RwLock<HashMap<String, WindowId>>>,
    /// 事件代理
    pub(crate) proxy: Arc<RwLock<Option<EventLoopProxy<UserEvent>>>>,
    /// 未运行前的创建窗口
    pub(crate) pending: RefCell<Vec<WindowConfig>>,
    /// 当前具有焦点的窗口
    pub(crate) curr: Arc<Mutex<Option<WindowId>>>,
    /// 回调
    pub(crate) listeners: Arc<RwLock<Listeners>>,
}

#[derive(Default)]
struct Listeners {
    on_close: Option<Box<dyn FnOnce() + Send + 'static>>,
    on_setup: Option<Box<dyn FnOnce() + Send + 'static>>,
}

unsafe impl Send for WindowManager {}
unsafe impl Sync for WindowManager {}

impl WindowManager {
    /// 判断当前是否有窗口
    pub fn is_empty(&self) -> bool {
        self.ids.read().is_empty()
    }

    /// 查找窗口id
    pub(crate) fn find_id(&self, label: &str) -> Option<WindowId> {
        self.ids.read().get(label).cloned()
    }

    pub fn find_window<'a>(&'a self, label: &'a str) -> Option<Window<'a>> {
        self.find_id(label).map(|id| self.new_window(id, label))
    }

    pub fn curr<'a>(&'a self) -> Option<Window<'a>> {
        let curr = { self.curr.lock().as_ref().cloned() }?;
        let label = { self.wms.read().get(&curr)?.label.clone() };
        let window = Window {
            id: curr,
            label,
            wm: self,
        };
        Some(window)
    }

    #[inline]
    pub(crate) fn new_window<'a>(&'a self, id: WindowId, label: impl ToString) -> Window<'a> {
        Window {
            id,
            label: label.to_string(),
            wm: self,
        }
    }

    /// 创建窗口
    pub fn create(&self, config: WindowConfig) -> Result<&Self> {
        let p = self.proxy.read();
        if let Some(proxy) = { p.as_ref() } {
            proxy.send_event(UserEvent::Create(config))?;
        } else {
            self.pending.borrow_mut().push(config);
        }
        Ok(self)
    }

    /// 退出窗口系统
    pub fn exit(&self) -> Result<()> {
        {
            let proxy = self.proxy.read();
            if let Some(proxy) = proxy.as_ref() {
                proxy.send_event(UserEvent::Exit)?;
            }
        }
        Ok(())
    }

    /// 运行窗口系统，运行前的事件会在运行后触发
    pub fn run(&self) -> ! {
        WindowRunner::new(self.clone()).run()
    }

    /// 窗口运行后的关闭回调
    pub fn on_close<F>(&self, on_close: F) -> &Self
    where
        F: FnOnce() + Send + 'static,
    {
        self.listeners.write().on_close = Some(Box::new(on_close));
        self
    }

    pub fn on_setup<F>(&self, on_setup: F) -> &Self
    where
        F: FnOnce() + Send + 'static,
    {
        self.listeners.write().on_setup = Some(Box::new(on_setup));
        self
    }

    /// 窗口运行后，设置事件代理
    pub(crate) fn setup_proxy(&self, proxy: EventLoopProxy<UserEvent>) {
        *self.proxy.write() = Some(proxy);
    }

    /// 窗口添加成功，添加到管理器中
    pub(crate) fn insert_created_window(&self, w_ref: WindowRef) {
        let id = w_ref.id;
        let label = w_ref.label.clone();
        {
            self.wms.write().insert(id, w_ref);
        }
        {
            self.ids.write().insert(label, id);
        }
    }

    pub(crate) fn set_curr_focused(&self, id: WindowId, focused: bool) {
        {
            let mut curr = self.curr.lock();
            if focused {
                *curr = Some(id);
            }
            // else if *curr == Some(id) {
            //     *curr = None;
            // }
        }
    }
}

impl WindowManager {
    #[inline]
    pub fn create_with_name(&self, name: impl ToString) -> Result<&Self> {
        self.create(WindowConfig::new(name.to_string()))
    }

    #[inline]
    pub fn create_with_url(&self, name: impl ToString, url: impl ToString) -> Result<&Self> {
        self.create(WindowConfig::new(name.to_string()).with_url(url.to_string()))
    }

    #[inline]
    pub fn create_with_html(&self, name: impl ToString, html: impl ToString) -> Result<&Self> {
        self.create(WindowConfig::new(name.to_string()).with_html(html.to_string()))
    }

    #[inline]
    pub fn create_new(&self, name: impl ToString) -> Result<&Self> {
        self.create(WindowConfig::new(name.to_string()))
    }
}

/// 持有的窗口对象
///
/// 当该对象被销毁时，会自动关闭窗口；反之，需要丢弃对象
struct WindowRef {
    id: WindowId,
    label: String,
    window: TaoWindow,
    webview: WryWebView,
}

/// 对外暴露的窗口对象
pub struct Window<'a> {
    id: WindowId,
    pub label: String,
    wm: &'a WindowManager,
}

impl<'a> Window<'a> {
    /// 关闭窗口
    pub fn close(&self) -> Option<()> {
        self.wm.close(self.id)
    }

    pub fn hide(&self) -> Option<()> {
        self.wm.hide(self.id)
    }

    pub fn show(&self) -> Option<()> {
        self.wm.show(self.id)
    }

    pub fn is_show(&self) -> Option<bool> {
        self.wm.is_show(self.id)
    }
}

pub trait WindowFindExt<ID, P, R> {
    fn find(self, id: ID, find: impl FnOnce(&P) -> R) -> Option<R>;
}

pub trait WindowOpExt<ID> {
    fn close(self, id: ID) -> Option<()>;
    fn hide(self, id: ID) -> Option<()>;
    fn show(self, id: ID) -> Option<()>;
    fn is_show(&self, id: ID) -> Option<bool>;
}

impl<R> WindowFindExt<WindowId, WryWebView, R> for &WindowManager {
    #[inline]
    fn find(self, id: WindowId, find: impl FnOnce(&WryWebView) -> R) -> Option<R> {
        self.wms.read().get(&id).map(|e| find(&e.webview))
    }
}

impl<R> WindowFindExt<&str, WryWebView, R> for &WindowManager {
    #[inline]
    fn find(self, id: &str, find: impl FnOnce(&WryWebView) -> R) -> Option<R> {
        let id = self.find_id(id)?;
        self.find(id, find)
    }
}

impl<R> WindowFindExt<WindowId, WindowRef, R> for &WindowManager {
    #[inline]
    fn find(self, id: WindowId, find: impl FnOnce(&WindowRef) -> R) -> Option<R> {
        self.wms.read().get(&id).map(find)
    }
}

impl<R> WindowFindExt<WindowId, TaoWindow, R> for &WindowManager {
    #[inline]
    fn find(self, id: WindowId, find: impl FnOnce(&TaoWindow) -> R) -> Option<R> {
        self.wms.read().get(&id).map(|e| find(&e.window))
    }
}

impl<R> WindowFindExt<&str, TaoWindow, R> for &WindowManager {
    #[inline]
    fn find(self, id: &str, find: impl FnOnce(&TaoWindow) -> R) -> Option<R> {
        let id = self.find_id(id)?;
        self.find(id, find)
    }
}

impl WindowOpExt<&str> for &WindowManager {
    fn close(self, id: &str) -> Option<()> {
        if let Some(id) = { self.ids.write().remove(id) } {
            {
                self.wms.write().remove(&id);
            }
        }
        Some(())
    }

    fn hide(self, id: &str) -> Option<()> {
        self.hide(self.find_id(id)?)
    }

    fn show(self, id: &str) -> Option<()> {
        self.show(self.find_id(id)?)
    }

    fn is_show(&self, id: &str) -> Option<bool> {
        self.is_show(self.find_id(id)?)
    }
}

impl WindowOpExt<WindowId> for &WindowManager {
    fn close(self, id: WindowId) -> Option<()> {
        if let Some(w_ref) = { self.wms.write().remove(&id) } {
            {
                self.ids.write().remove(&w_ref.label);
            }
        }
        Some(())
    }

    fn hide(self, id: WindowId) -> Option<()> {
        self.find(id, |e: &TaoWindow| e.set_visible(false))
    }

    fn show(self, id: WindowId) -> Option<()> {
        self.find(id, |e: &TaoWindow| e.set_visible(true))
    }

    fn is_show(&self, id: WindowId) -> Option<bool> {
        self.find(id, |e: &TaoWindow| e.is_visible())
    }
}
