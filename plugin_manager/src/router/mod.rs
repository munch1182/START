use crate::{router::apiv1::ApiV1, urlpath::UrlPath};
use axum::{Router, response::IntoResponse, routing::get};
use std::{
    cell::RefCell,
    sync::{Arc, OnceLock},
};

mod admin;
mod apiv1;

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

#[derive(Clone)]
pub struct AppState {}

unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}

impl AppState {
    fn new() -> Self {
        Self {}
    }
}
