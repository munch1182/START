use axum::{Router, response::IntoResponse, routing::get};
use libcommon::prelude::info;

use crate::urlpath::UrlPath;

pub struct Admin {}

impl Admin {
    pub fn new() -> Self {
        Self {}
    }

    pub fn router(&self, path: &UrlPath) -> Router {
        let scan = path.new_path_with("/scan");
        let list = path.new_path_with("/list");

        info!("admin router: {}", scan.all_path());
        info!("admin router: {}", list.all_path());
        Router::new()
            .route(scan.curr_part(), get(Self::admin))
            .route(list.curr_part(), get(Self::admin))
    }

    async fn admin() -> impl IntoResponse {
        "Hello, world!"
    }
}
