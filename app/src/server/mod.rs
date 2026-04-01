use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
    routing::get,
};
use dashmap::DashMap;
use libcommon::{Result, debug};
#[cfg(any(not(debug_assertions), feature = "use-embed"))]
use libcommon::{trace, warn};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

#[cfg(any(not(debug_assertions), feature = "use-embed"))]
use rust_embed::RustEmbed;

pub type RouteTable = Arc<DashMap<String, RouteSource>>;

/// 静态文件服务器配置
#[derive(Debug, Clone)]
pub struct Server {
    pub port: u16,
    pub static_routes: Vec<StaticRoute>, // 静态宿主路由（编译时确定）
    pub plugin_routes: RouteTable,       // 动态插件路由表
}

#[derive(Debug, Clone)]
pub struct StaticRoute {
    pub url_route: String,
    pub source: RouteSource,
}

#[cfg(all(debug_assertions, not(feature = "use-embed")))]
const DEV_SERVER_URL: &str = "http://localhost:3000/";

/// 嵌入资源（发布模式下的宿主前端）
#[cfg(any(not(debug_assertions), feature = "use-embed"))]
#[derive(RustEmbed)]
#[folder = "dist/"]
pub struct ServerAssets;

/// 数据来源
#[derive(Debug, Clone)]
pub enum RouteSource {
    /// 文件系统来源（用于插件 UI）
    File { path: String },
    /// 嵌入资源来源（用于宿主前端）
    #[cfg(any(not(debug_assertions), feature = "use-embed"))]
    Embedded,
}

impl Server {
    /// 根据当前编译模式创建服务器
    pub fn new(port: u16) -> Self {
        let static_routes = {
            #[cfg(any(not(debug_assertions), feature = "use-embed"))]
            {
                // 宿主前端使用嵌入资源
                vec![StaticRoute::new_embedded("/")]
            }
            #[cfg(all(debug_assertions, not(feature = "use-embed")))]
            {
                // 开发模式：无静态宿主路由，主窗口直接连接 dev server
                vec![]
            }
        };
        Self {
            port,
            static_routes,
            plugin_routes: Arc::new(DashMap::new()),
        }
    }

    /// 返回主窗口应加载的 URL（开发模式返回 dev server，发布模式返回内嵌服务器地址）
    pub fn window_url(&self) -> String {
        #[cfg(any(not(debug_assertions), feature = "use-embed"))]
        {
            format!("http://127.0.0.1:{}/", self.port)
        }
        #[cfg(all(debug_assertions, not(feature = "use-embed")))]
        {
            DEV_SERVER_URL.to_string()
        }
    }

    /// 返回静态文件服务器自身地址（用于插件 UI 等内部访问）
    pub fn server_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    /// 动态添加插件路由
    pub fn add_plugin_route(&self, plugin_id: &str, base_dir: impl Into<String>) {
        let url_route = format!("/plugins/{plugin_id}");
        let source = RouteSource::File {
            path: base_dir.into(),
        };
        debug!("Adding plugin route: {url_route} -> {source:?}");
        self.plugin_routes.insert(url_route, source);
    }

    /// 移除插件路由
    pub fn remove_plugin_route(&self, plugin_id: &str) {
        let url_route = format!("/plugins/{plugin_id}");
        debug!("Removing plugin route: {url_route}");
        self.plugin_routes.remove(&url_route);
    }

    /// 启动服务器
    pub async fn run(mut self) -> Result<()> {
        #[cfg(all(debug_assertions, not(feature = "use-embed")))]
        self.start_dev_server();

        let mut app = Router::new().route("/health", get(health_check));

        // ---------- 1. 静态宿主路由 ----------
        for route in &self.static_routes {
            match &route.source {
                RouteSource::File { path } => {
                    let serve_dir = ServeDir::new(path).not_found_service(get(not_found));
                    if route.url_route == "/" {
                        app = app.fallback_service(serve_dir);
                    } else {
                        app = app.nest_service(&route.url_route, serve_dir);
                    }
                    debug!("  [Static FS] {} -> {}", route.url_route, path);
                }
                #[cfg(any(not(debug_assertions), feature = "use-embed"))]
                RouteSource::Embedded => {
                    let embedded_router = Self::embedded_router();
                    if route.url_route == "/" {
                        app = app.merge(embedded_router);
                    } else {
                        app = app.nest(&route.url_route, embedded_router);
                    }
                    debug!("  [Static Embedded] {}", route.url_route);
                }
            }
        }

        // ---------- 2. 插件动态路由 ----------
        let plugin_routes = self.plugin_routes.clone();
        let plugin_fallback = Router::new()
            .fallback(move |req| serve_plugin(req, plugin_routes.clone()))
            .layer(CorsLayer::new().allow_origin(Any));
        app = app.nest("/plugins", plugin_fallback);

        app = app.layer(CorsLayer::new().allow_origin(Any));

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        self.port = listener.local_addr()?.port();
        debug!("Starting static file server at {}", self.server_url());
        axum::serve(listener, app).await?;
        Ok(())
    }

    #[cfg(all(debug_assertions, not(feature = "use-embed")))]
    fn start_dev_server(&self) {
        tokio::spawn(async {
            use libcommon::warn;
            debug!("Starting frontend dev server...");
            #[cfg(target_os = "windows")]
            let result = std::process::Command::new("cmd")
                .current_dir("./app")
                .args(["/c", "pnpm", "run", "dev"])
                .output();
            #[cfg(not(target_os = "windows"))]
            let result = std::process::Command::new("pnpm")
                .current_dir("./app")
                .arg("run")
                .arg("dev")
                .output();

            match result {
                Ok(output) => {
                    if output.status.success() {
                        debug!("Frontend dev server started successfully");
                    } else {
                        warn!("Frontend dev server failed: {:?}", output);
                    }
                }
                Err(e) => warn!("Failed to start frontend dev server: {}", e),
            }
        });
    }

    #[cfg(any(not(debug_assertions), feature = "use-embed"))]
    fn embedded_router() -> Router {
        Router::new().fallback(move |req| serve_embedded::<ServerAssets>(req))
    }
}

// ---------- 插件动态服务 ----------
async fn serve_plugin(req: Request<Body>, routes: RouteTable) -> impl IntoResponse {
    let path = req.uri().path().to_string();
    debug!("Plugin request: {}", path);

    // 寻找最长匹配前缀（键如 "/plugins/foo"）
    let best_match = routes
        .iter()
        .filter(|entry| path.starts_with(entry.key().as_str()))
        .max_by_key(|entry| entry.key().len());

    let (prefix, source) = match best_match {
        Some(entry) => (entry.key().clone(), entry.value().clone()),
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    // 提取相对路径
    let relative = path.strip_prefix(&prefix).unwrap().trim_start_matches('/');

    match source {
        RouteSource::File { path: base_dir } => serve_file(&base_dir, relative).await,
        #[cfg(any(not(debug_assertions), feature = "use-embed"))]
        RouteSource::Embedded => {
            // 理论上插件不应使用 Embedded，但可留作扩展
            StatusCode::NOT_IMPLEMENTED.into_response()
        }
    }
}

/// 从文件系统服务插件文件
async fn serve_file(base_dir: &str, relative: &str) -> Response {
    use std::path::PathBuf;

    let full_path = PathBuf::from(base_dir).join(relative);
    // 安全检查：防止目录穿越
    if !full_path.starts_with(base_dir) {
        return StatusCode::FORBIDDEN.into_response();
    }

    let file_path = if relative.is_empty() {
        full_path.join("index.html")
    } else {
        full_path
    };

    match tokio::fs::read(&file_path).await {
        Ok(content) => {
            let mime = mime_guess::from_path(&file_path).first_or_octet_stream();
            Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

impl StaticRoute {
    pub fn new_file(url_route: &str, path: impl Into<String>) -> Self {
        Self {
            url_route: url_route.into(),
            source: RouteSource::File { path: path.into() },
        }
    }

    #[cfg(any(not(debug_assertions), feature = "use-embed"))]
    pub fn new_embedded(url_route: &str) -> Self {
        Self {
            url_route: url_route.into(),
            source: RouteSource::Embedded,
        }
    }
}

/// 嵌入资源处理器（用于宿主前端）
#[cfg(any(not(debug_assertions), feature = "use-embed"))]
async fn serve_embedded<E: RustEmbed>(req: Request<Body>) -> impl IntoResponse {
    let path = req.uri().path();
    let relative_path = path.trim_start_matches('/');

    let file_path = if relative_path.is_empty() {
        "index.html"
    } else {
        relative_path
    };

    let resp = match E::get(file_path) {
        Some(content) => {
            let mime = mime_guess::from_path(file_path).first_or_octet_stream();
            Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        None => StatusCode::NOT_FOUND.into_response(),
    };
    let status = resp.status().as_u16();
    if status == 200 {
        trace!("Embedded request: {path} -> {file_path}: {status}");
    } else {
        warn!("Embedded request: {path} -> {file_path}: {status}");
    }
    resp
}

async fn health_check() -> &'static str {
    "ok"
}

async fn not_found() -> &'static str {
    "Not Found"
}
