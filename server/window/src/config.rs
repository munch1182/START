use libcommon::With;
use std::sync::Arc;
use tao::{
    dpi::{PhysicalPosition, PhysicalSize},
    window::WindowBuilder,
};
use wry::WebViewBuilder;

type WindowBuilderFn = dyn Fn(WindowBuilder) -> WindowBuilder + Send + Sync;
type WebViewBuilderFn = dyn Fn(WebViewBuilder) -> WebViewBuilder + Send + Sync;

#[derive(Clone, With)]
pub struct WindowConfig {
    /// 窗口标题和标签
    pub title: String,
    /// 窗口加载的URL
    pub url: Option<String>,
    /// 本地文件或者html字符串
    pub html: Option<String>,
    /// 窗口大小
    pub size: Option<(i32, i32)>,
    /// 窗口位置
    pub postion: Option<(i32, i32)>,
    /// 暴露自定义的窗口构建函数
    #[with(skip)]
    pub with_window: Option<Arc<WindowBuilderFn>>,
    /// 暴露自定义的WebView构建函数
    #[with(skip)]
    pub with_webview: Option<Arc<WebViewBuilderFn>>,
}

impl WindowConfig {
    pub fn new(label: impl ToString) -> Self {
        Self {
            title: label.to_string(),
            url: None,
            html: None,
            size: None,
            postion: None,
            with_window: None,
            with_webview: None,
        }
    }

    #[inline]
    pub fn with_window<F>(mut self, f: F) -> Self
    where
        F: Fn(WindowBuilder) -> WindowBuilder + 'static + Send + Sync,
    {
        self.with_window = Some(Arc::new(f));
        self
    }

    #[inline]
    pub fn with_webview<F>(mut self, f: F) -> Self
    where
        F: Fn(WebViewBuilder) -> WebViewBuilder + 'static + Send + Sync,
    {
        self.with_webview = Some(Arc::new(f));
        self
    }

    pub(crate) fn build_window(&self, wb: WindowBuilder) -> WindowBuilder {
        let mut build = wb.with_title(&self.title);
        if let Some((w, h)) = self.size {
            build = build.with_inner_size(PhysicalSize::new(w, h));
        }
        if let Some((x, y)) = self.postion {
            build = build.with_position(PhysicalPosition::new(x, y));
        }
        if let Some(f) = &self.with_window {
            build = f(build);
        }
        build
    }

    pub(crate) fn build_webview<'a>(&'a self, build: WebViewBuilder<'a>) -> WebViewBuilder<'a> {
        let mut build = build;
        if let Some(url) = &self.url {
            build = build.with_url(url);
        }
        if let Some(html) = &self.html {
            build = build.with_html(html);
        }
        if let Some(f) = &self.with_webview {
            build = f(build);
        }
        build
    }
}
