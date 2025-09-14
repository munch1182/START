use std::sync::Arc;

pub use tao::window::WindowBuilder;
pub use wry::WebViewBuilder;

pub type WindowBuilderFn = dyn Fn(WindowBuilder) -> WindowBuilder + Send + Sync;
pub type WebViewBuilderFn = dyn Fn(WebViewBuilder) -> WebViewBuilder + Send + Sync;

#[derive(Clone)]
pub struct WindowConfig {
    pub label: String,
    pub url: Option<String>,  // url地址
    pub html: Option<String>, // 本地文件或者html字符串
    pub size: Option<(i32, i32)>,
    pub position: Option<(i32, i32)>,
    pub with_window: Option<Arc<WindowBuilderFn>>,
    pub with_webview: Option<Arc<WebViewBuilderFn>>,
}

impl WindowConfig {
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
    pub fn new(label: &str) -> Self {
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

    #[inline]
    pub fn with_url(mut self, url: impl ToString) -> Self {
        self.url = Some(url.to_string());
        self
    }

    #[inline]
    pub fn with_html(mut self, html: impl ToString) -> Self {
        self.html = Some(html.to_string());
        self
    }

    #[inline]
    pub fn with_size(mut self, width: i32, height: i32) -> Self {
        self.size = Some((width, height));
        self
    }

    #[inline]
    pub fn with_position(mut self, x: i32, y: i32) -> Self {
        self.position = Some((x, y));
        self
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
