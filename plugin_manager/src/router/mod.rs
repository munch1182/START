use crate::{router::apiv1::ApiV1, urlpath::UrlPath};
use axum::{Router, routing::get};
use libcommon::{
    prelude::{Result, info},
    record,
};
use serde::Serialize;
use std::{
    cell::RefCell,
    sync::{Arc, OnceLock},
};

mod admin;
mod apiv1;
mod appstate;
mod plugin;
pub use appstate::{AppConfig, AppState, GetExt};

pub(crate) trait ApiImpl<'a> {
    fn new(parent: &UrlPath<'a>) -> Self;
    fn router_str(&self) -> String;
    fn router(&self) -> Router<Arc<AppState>>;
}

pub(crate) static APP_STATE: OnceLock<Arc<AppState>> = OnceLock::new();

pub struct AppRouter<'a> {
    path: RefCell<UrlPath<'a>>,
}

impl<'a> AppRouter<'a> {
    pub fn new(server: &'a str) -> Self {
        let path = UrlPath::new(server);
        Self {
            path: RefCell::new(path),
        }
    }

    pub fn router(&self, config: AppConfig) -> Router {
        #[cfg(debug_assertions)]
        let _ = write_http_clear();
        let state = Arc::new(AppState::new(config));
        let _ = APP_STATE.set(state);
        let v1 = ApiV1::new(&self.path.borrow());
        Router::new()
            .route("/", get(no_router))
            .nest(&v1.router_str(), v1.router())
            .with_state(APP_STATE.wait().clone())
    }
}

async fn no_router() -> &'static str {
    "Hello, world!"
}

pub(crate) fn info_router(path: &UrlPath) {
    #[cfg(debug_assertions)]
    let _ = write_http(&path.all_path());
    record!("router: {}", path.all_path());
}

#[allow(unused)]
pub(crate) fn info_router_with(path: &UrlPath, query: impl Serialize) {
    #[cfg(debug_assertions)]
    let _ = write_http_with(
        &path.all_path(),
        &serde_json::to_string_pretty(&query).unwrap_or_default(),
    );
    record!("router: {}", path.all_path());
}

pub(crate) fn info_router_with_query(path: &UrlPath, query: impl Serialize) {
    #[cfg(debug_assertions)]
    {
        let query = serde_urlencoded::to_string(&query).unwrap_or_default();
        let _ = write_http(&format!("{}?{query}", path.all_path()));
    }
    record!("router: {}", path.all_path());
}

#[cfg(debug_assertions)]
fn write_http_clear() -> Result<()> {
    use libcommon::curr_dir;
    use std::fs;
    let file = curr_dir!("test_router.http")?;
    fs::remove_file(file)?;
    Ok(())
}

#[cfg(debug_assertions)]
fn write_http(path: &str) -> Result<()> {
    use libcommon::{curr_dir, ext::WriteAppendExt};

    let mut file = curr_dir!("test_router.http")?;
    let _ = file.write_append(format!("GET {path}\n\n###\n\n").as_bytes());
    Ok(())
}

#[cfg(debug_assertions)]
fn write_http_with(path: &str, content: &str) -> Result<()> {
    use libcommon::{curr_dir, ext::WriteAppendExt};

    let mut file = curr_dir!("test_router.http")?;
    let _ = file.write_append(format!("GET {path}\n{content}\n\n###\n\n").as_bytes());
    Ok(())
}
