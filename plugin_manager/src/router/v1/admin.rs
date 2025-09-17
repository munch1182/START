use crate::{
    pm::PluginId,
    respres::RespResult,
    router::{ApiImpl, AppState},
    urlpath::UrlPath,
};
use axum::{
    Router,
    extract::{Query, State},
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
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
        let del_p = self.path.borrow().new_path_with("/del");

        Router::new()
            .route(scan_p.router_str(), get(scan))
            .route(list_p.router_str(), get(list))
            .route(config_p.router_str(), get(config))
            .route(del_p.router_str(), get(del))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DelReq {
    pub id: String,
}

async fn del(State(app): State<Arc<AppState>>, Query(id): Query<DelReq>) -> RespResult<bool> {
    app.pm().remove(&PluginId::new_by(id.id)).into()
}

async fn scan(State(app): State<Arc<AppState>>) -> RespResult<usize> {
    let plugins = app.pm().scan();
    plugins.len().into()
}

async fn list(State(app): State<Arc<AppState>>) -> RespResult<Vec<Value>> {
    app.pm()
        .info()
        .iter()
        .map(|(id, p)| {
            json!({
                "name": p.name,
                "version": p.version,
                "id": id.as_str(),
            })
        })
        .collect::<Vec<_>>()
        .into()
}

async fn config(State(app): State<Arc<AppState>>) -> RespResult<Value> {
    app.config_str().into()
}
