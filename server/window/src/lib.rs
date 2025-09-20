mod config;
mod event;
mod key;
mod runner;

use crate::runner::WindowRunner;
pub use config::*;
pub use event::*;
pub use key::*;
use libcommon::prelude::*;
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, RwLock},
};
pub use tao::window::Window as TaoWindow;
use tao::{event_loop::EventLoopProxy, window::WindowId};
pub use wry::WebView as WryWebView;

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
    /// 按键管理: 当前问题：当按下tab切换窗口后，将无法获取到按键事件
    pub(crate) key: Arc<RwLock<KeyHelper>>,
}

unsafe impl Send for WindowManager {}
unsafe impl Sync for WindowManager {}

impl WindowManager {
    /// 判断当前是否有窗口
    pub fn is_empty(&self) -> bool {
        self.ids.read().map(|e| e.is_empty()).unwrap_or_default()
    }

    /// 查找窗口id
    pub(crate) fn find_id(&self, label: &str) -> Option<WindowId> {
        self.ids.read().newerr().ok()?.get(label).cloned()
    }

    pub fn find_window<'a>(&'a self, f: &'a str) -> Option<Window<'a>> {
        self.find_id(f).map(|id| self.new_window(id, f))
    }

    #[inline]
    pub(crate) fn new_window<'a>(&'a self, id: WindowId, label: &'a str) -> Window<'a> {
        Window {
            id,
            label,
            wm: self,
        }
    }

    /// 创建窗口
    pub fn create(&self, config: WindowConfig) -> Result<&Self> {
        let p = self.proxy.read().newerr()?;
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
            let proxy = self.proxy.read().newerr()?;
            if let Some(proxy) = proxy.as_ref() {
                proxy.send_event(UserEvent::Exit)?;
            }
        }
        Ok(())
    }

    /// 运行窗口系统，运行前的事件会在运行后触发
    pub fn run(&self) -> ! {
        if let Ok(key) = self.key.read() {
            key.set_callback()
        }
        WindowRunner::new(self.clone()).run()
    }

    /// 窗口运行后，设置事件代理
    pub(crate) fn setup_proxy(&self, proxy: EventLoopProxy<UserEvent>) {
        if let Ok(mut p) = self.proxy.write() {
            *p = Some(proxy);
        }
    }

    /// 窗口添加成功，添加到管理器中
    pub(crate) fn insert(&self, w_ref: WindowRef) -> Result<()> {
        let id = w_ref.id;
        let label = w_ref.label.clone();
        {
            self.wms.write().newerr()?.insert(id, w_ref);
        }
        {
            self.ids.write().newerr()?.insert(label, id);
        }
        Ok(())
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
    pub label: &'a str,
    wm: &'a WindowManager,
}

impl<'a> Window<'a> {
    /// 关闭窗口
    pub fn close(&self) -> Option<()> {
        self.wm.close(self.id)
    }
}

pub trait WindowFindExt<ID, P, R> {
    fn find(self, id: ID, find: impl FnOnce(&P) -> R) -> Option<R>;
}

pub trait WindowOpExt<ID> {
    fn close(self, id: ID) -> Option<()>;
}

impl<R> WindowFindExt<WindowId, WryWebView, R> for &WindowManager {
    #[inline]
    fn find(self, id: WindowId, find: impl FnOnce(&WryWebView) -> R) -> Option<R> {
        self.wms
            .read()
            .ok()
            .and_then(|e| e.get(&id).map(|e| find(&e.webview)))
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
        self.wms.read().ok().and_then(|e| e.get(&id).map(find))
    }
}

impl<R> WindowFindExt<WindowId, TaoWindow, R> for &WindowManager {
    #[inline]
    fn find(self, id: WindowId, find: impl FnOnce(&TaoWindow) -> R) -> Option<R> {
        self.wms
            .read()
            .ok()
            .and_then(|e| e.get(&id).map(|e| find(&e.window)))
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
        if let Some(id) = { self.ids.write().ok()?.remove(id) } {
            {
                self.wms.write().ok()?.remove(&id);
            }
        }
        Some(())
    }
}

impl WindowOpExt<WindowId> for &WindowManager {
    fn close(self, id: WindowId) -> Option<()> {
        if let Some(w_ref) = { self.wms.write().ok()?.remove(&id) } {
            {
                self.ids.write().ok()?.remove(&w_ref.label);
            }
        }
        Some(())
    }
}

impl KeyListenerExt for &WindowManager {
    #[inline]
    fn check(self, key: String) -> bool {
        self.key.read().ok().map(|e| e.check(key)).unwrap_or(false)
    }

    #[inline]
    fn register_key_listener(self, key: String, lis: impl Fn(&str) + 'static) -> Result<Self> {
        self.key.write().newerr()?.register_key_listener(key, lis)?;
        Ok(self)
    }

    #[inline]
    fn unregister_key_listener(self, key: &str) -> Result<Self> {
        self.key.write().newerr()?.unregister_key_listener(key)?;
        Ok(self)
    }
}
