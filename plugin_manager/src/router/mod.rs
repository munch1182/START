mod admin;

use axum::{Router, response::IntoResponse, routing::get};
use std::sync::Arc;

use crate::{router::admin::Admin, urlpath::UrlPath};

pub fn router(path: &mut UrlPath) -> Router {
    path.push("/api/v1");

    let state = Arc::new(AppState::new());
    let admin = Admin::new();

    let router_prefix =
        Router::new().nest(path.new_path_with("/admin").curr_part(), admin.router(path));

    Router::new()
        .with_state(state)
        .fallback(get(no_router))
        .nest(path.curr_part(), router_prefix)
}

async fn no_router() -> impl IntoResponse {
    "Hello, world!"
}

struct AppState {}

impl AppState {
    fn new() -> Self {
        Self {}
    }
}
