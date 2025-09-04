use crate::{router::apiv1::ApiV1, urlpath::UrlPath};
use axum::{Router, response::IntoResponse, routing::get};
use libcommon::prelude::{Result, info};
use std::{
    cell::RefCell,
    sync::{Arc, OnceLock},
};

mod admin;
mod apiv1;
mod plugin;

pub(crate) trait ApiImpl<'a> {
    fn new(parent: &UrlPath<'a>) -> Self;
    fn router_str(&self) -> String;
    fn router(&self) -> Router<Arc<AppState>>;
}

static APP_STATE: OnceLock<Arc<AppState>> = OnceLock::new();

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

    pub fn router(&self) -> Router {
        #[cfg(debug_assertions)]
        let _ = write_http_clear();
        let state = Arc::new(AppState::new());
        let _ = APP_STATE.set(state);
        let v1 = ApiV1::new(&self.path.borrow());
        Router::new()
            .route("/", get(no_router))
            .nest(&v1.router_str(), v1.router())
            .with_state(APP_STATE.wait().clone())
    }
}

async fn no_router() -> impl IntoResponse {
    "Hello, world!"
}

pub(crate) fn info_router(path: &UrlPath) {
    #[cfg(debug_assertions)]
    let _ = write_http(&path.all_path());
    info!("router: {}", path.all_path());
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

#[derive(Clone)]
pub struct AppState {}

unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}

impl AppState {
    fn new() -> Self {
        Self {}
    }
}
