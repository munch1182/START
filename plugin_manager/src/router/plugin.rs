use crate::{
    pm::PluginId,
    respres::RespResult,
    router::{ApiImpl, AppState},
    urlpath::UrlPath,
};
use axum::{
    Router,
    body::Body,
    extract::{Path, State},
    http::Request,
    routing::{any, get},
};
use libcommon::newerr;
use plugin_d::PluginInfo;
use serde_json::Value;
use std::{cell::RefCell, sync::Arc};

pub(crate) struct Plugin<'a> {
    prefix: &'a str,
    _path: RefCell<UrlPath<'a>>,
}

impl<'a> ApiImpl<'a> for Plugin<'a> {
    fn new(parent: &UrlPath<'a>) -> Self {
        let prefix = "/plugin";
        Self {
            prefix,
            _path: RefCell::new(parent.new_path_with(prefix)),
        }
    }

    fn router_str(&self) -> String {
        self.prefix.to_string()
    }

    fn router(&self) -> Router<Arc<AppState>> {
        Router::new()
            .route("/{id}", get(plugin_info))
            .route("/{id}/{*query}", any(plugin))
    }
}

async fn plugin_info(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> RespResult<PluginInfo> {
    let plugin_id = PluginId::new_by(&id);
    state
        .pm()
        .get(plugin_id)
        .ok_or(newerr!("plugin {} not found", id))
        .into()
}

async fn plugin(
    State(state): State<Arc<AppState>>,
    Path((id, path)): Path<(String, String)>,
    req: Request<Body>,
) -> RespResult<Value> {
    let plugin_id = PluginId::new_by(id);
    let resp = state.pm().handle(&plugin_id, path, req);
    RespResult::from(resp)
}
