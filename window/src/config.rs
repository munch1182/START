use libcommon::prelude::With;
use std::sync::Arc;
pub use tao::window::WindowBuilder;
pub use wry::WebViewBuilder;

pub type WindowBuilderFn = dyn Fn(WindowBuilder) -> WindowBuilder + Send + Sync;
pub type WebViewBuilderFn = dyn Fn(WebViewBuilder) -> WebViewBuilder + Send + Sync;

#[derive(Clone, With)]
pub struct WindowConfig {
    pub label: String,
    pub url: Option<String>,  // url地址
    pub html: Option<String>, // 本地文件或者html字符串
    pub size: Option<(i32, i32)>,
    pub position: Option<(i32, i32)>,
    #[with(skip)]
    pub with_window: Option<Arc<WindowBuilderFn>>,
    #[with(skip)]
    pub with_webview: Option<Arc<WebViewBuilderFn>>,
}

impl WindowConfig {
    pub fn new(label: impl ToString) -> Self {
        Self {
            label: label.to_string(),
            url: None,
            html: None,
            size: None,
            position: None,
            with_window: None,
            with_webview: None,
        }
    }

    pub(crate) fn build_window(&self, build: WindowBuilder) -> WindowBuilder {
        let mut build = build.with_title(&self.label);
        if let Some(s) = &self.size {
            build = build.with_inner_size(tao::dpi::PhysicalSize::new(s.0, s.1));
        }
        if let Some(p) = &self.position {
            build = build.with_position(tao::dpi::PhysicalPosition::new(p.0, p.1));
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
}
