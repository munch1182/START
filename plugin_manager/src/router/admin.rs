use crate::{
    router::{ApiImpl, AppState, info_router},
    urlpath::UrlPath,
};
use axum::{Json, Router, response::IntoResponse, routing::get};
use serde_json::json;
use std::{cell::RefCell, sync::Arc};

pub(crate) struct Admin<'a> {
    prefix: &'a str,
    path: RefCell<UrlPath<'a>>,
}

impl<'a> ApiImpl<'a> for Admin<'a> {
    fn new(parent: &UrlPath<'a>) -> Self {
        let prefix = "/admin";
        Self {
            prefix,
            path: RefCell::new(parent.new_path_with(prefix)),
        }
    }

    fn router_str(&self) -> String {
        self.prefix.to_string()
    }

    fn router(&self) -> Router<Arc<AppState>> {
        let scan_p = self.path.borrow().new_path_with("/scan");
        let list_p = self.path.borrow().new_path_with("/list");

        info_router(&scan_p);
        info_router(&list_p);

        Router::new()
            .route(scan_p.curr_part().unwrap_or_default(), get(scan))
            .route(list_p.curr_part().unwrap_or_default(), get(list))
    }
}

async fn scan() -> impl IntoResponse {
    "scan"
}
async fn list() -> impl IntoResponse {
    Json(json!({"a":1}))
}
