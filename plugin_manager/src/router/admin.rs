use crate::{
    respres::RespResult,
    router::{ApiImpl, AppState, info_router},
    urlpath::UrlPath,
};
use axum::{Router, extract::State, routing::get};
use serde_json::Value;
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
        let config_p = self.path.borrow().new_path_with("/config");

        info_router(&scan_p);
        info_router(&list_p);
        info_router(&config_p);

        Router::new()
            .route(scan_p.curr_part().unwrap_or_default(), get(scan))
            .route(list_p.curr_part().unwrap_or_default(), get(list))
            .route(config_p.curr_part().unwrap_or_default(), get(config))
    }
}

async fn scan(State(app): State<Arc<AppState>>) -> RespResult<usize> {
    app.pm().scan().into()
}

async fn list(State(app): State<Arc<AppState>>) -> RespResult<Vec<Value>> {
    app.pm()
        .list()
        .iter()
        .map(|p| serde_json::to_value(p).unwrap_or_default())
        .collect::<Vec<_>>()
        .into()
}

async fn config(State(app): State<Arc<AppState>>) -> RespResult<Value> {
    app.config_str().into()
}
