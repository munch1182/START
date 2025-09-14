use crate::{UserEvent, WindowConfig, runner::WindowRunner};
use libcommon::prelude::*;
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tao::{event_loop::EventLoopProxy, window::WindowId};

/// window对象可以执行的操作
pub trait WindowHandleExt<'a> {
    fn close(&self) -> Result<()>;
}

/// 窗口创建操作
pub trait WindowCreateExt {
    fn create_title(&self, label: &str) -> Result<&Self>;
    fn create_url(&self, label: &str, url: impl ToString) -> Result<&Self>;
    fn create_html(&self, label: &str, code: impl ToString) -> Result<&Self>;
    fn create(&self, config: WindowConfig) -> Result<&Self>;
}

/// wm可以执行的操作
pub trait WindowExecuteExt<'a, ID>: WindowCreateExt {
    fn run(&self) -> !;
    fn close(&self, id: ID) -> Result<()>;
    fn find(&'a self, id: ID) -> Option<Window<'a>>;
}

/// 窗口管理器
#[derive(Clone, Default)]
pub struct App {
    pub(crate) wm: Arc<RwLock<HashMap<WindowId, WindowHandle>>>,
    pub(crate) id: Arc<RwLock<HashMap<String, WindowId>>>,
    pub(crate) proxy: Arc<RwLock<Option<EventLoopProxy<UserEvent>>>>,
    // 在运行前创建的窗口，运行后立即创建
    pub(crate) pending: RefCell<Vec<WindowConfig>>,
}

unsafe impl Send for App {}
unsafe impl Sync for App {}

/// 持有的窗口相关对象
pub(crate) struct WindowHandle {
    pub(crate) id: WindowId,
    pub(crate) label: String,
    pub(crate) _window: tao::window::Window,
    pub(crate) _webview: wry::WebView,
}

unsafe impl Send for WindowHandle {}
unsafe impl Sync for WindowHandle {}

/// 对外提供的窗口对象
pub struct Window<'a> {
    id: WindowId,
    pub label: &'a str,
    wm: &'a App,
}

impl<'a> WindowHandleExt<'a> for Window<'a> {
    fn close(&self) -> Result<()> {
        self.wm.close_impl(self.id)
    }
}

impl WindowCreateExt for App {
    fn create_title(&self, label: &str) -> Result<&Self> {
        self.create_impl(WindowConfig::new(label))?;
        Ok(self)
    }

    /// 通过url地址创建窗口
    fn create_url(&self, label: &str, url: impl ToString) -> Result<&Self> {
        self.create_impl(WindowConfig::new(label).with_url(url.to_string()))?;
        Ok(self)
    }

    /// 通过本地html代码创建窗口
    fn create_html(&self, label: &str, code: impl ToString) -> Result<&Self> {
        self.create_impl(WindowConfig::new(label).with_html(code.to_string()))?;
        Ok(self)
    }

    fn create(&self, config: WindowConfig) -> Result<&Self> {
        self.create_impl(config)?;
        Ok(self)
    }
}

impl<'a> WindowExecuteExt<'a, &'a str> for App {
    fn run(&self) -> ! {
        WindowRunner::with_manager(self.clone()).run()
    }

    fn close(&self, label: &str) -> Result<()> {
        if let Some(w) = self.find(label) {
            w.close()?;
        }
        Ok(())
    }

    fn find(&'a self, label: &'a str) -> Option<Window<'a>> {
        if let Some(id) = self.id.read().ok()?.get(label) {
            let w = Window {
                id: *id,
                label,
                wm: self,
            };
            return Some(w);
        }
        None
    }
}

impl App {
    pub fn empty(&self) -> bool {
        if let Ok(wm) = self.wm.read() {
            return wm.is_empty();
        }
        false
    }

    pub(crate) fn close_impl(&self, id: WindowId) -> Result<()> {
        let w = self.wm.write().map_err_ext()?.remove(&id);
        if let Some(w) = w {
            self.id.write().map_err_ext()?.remove(&w.label);
        }
        Ok(())
    }

    pub(crate) fn add_wh(&self, wh: WindowHandle) -> Result<()> {
        info!("Add window({}({:?}))", wh.label, wh.id);
        self.id
            .write()
            .map_err_ext()?
            .insert(wh.label.clone(), wh.id);
        self.wm.write().map_err_ext()?.insert(wh.id, wh);
        Ok(())
    }

    /// 创建窗口
    /// 如果代理未设置，则将窗口配置保存到pending中，等待代理设置后立即创建
    /// 如果代理已设置，则发送创建事件
    fn create_impl(&self, wc: WindowConfig) -> Result<()> {
        if let Some(proxy) = self.proxy.read().map_err_ext()?.as_ref() {
            proxy.send_event(UserEvent::Create(wc))?;
        } else {
            self.pending.borrow_mut().push(wc);
        }
        Ok(())
    }
    pub(crate) fn setup_proxy(&self, proxy: EventLoopProxy<UserEvent>) {
        if let Ok(mut p) = self.proxy.write() {
            *p = Some(proxy);
            info!("Setup proxy");
        }
    }
}
