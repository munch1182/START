use crate::{router::v1::ApiV1, urlpath::UrlPath};
use axum::{Router, routing::get};
use std::{
    cell::RefCell,
    sync::{Arc, OnceLock},
};

mod appstate;
mod v1;
pub use appstate::{AppConfig, AppState};

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

    pub fn router(&self) -> Router {
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
