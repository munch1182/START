use crate::{
    respres::RespResult,
    router::{ApiImpl, AppState, info_router, plugin::Plugin},
    urlpath::UrlPath,
};
use axum::{Router, extract::State, routing::get};
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

        info_router(&scan_p);
        info_router(&list_p);
        info_router(&config_p);

        Router::new()
            .route(
                scan_p.curr_part().unwrap_or_default(),
                get({
                    let parent = self.path.borrow().parent().all_path();
                    move |app: State<Arc<AppState>>| scan(app, parent)
                }),
            )
            .route(list_p.curr_part().unwrap_or_default(), get(list))
            .route(config_p.curr_part().unwrap_or_default(), get(config))
    }
}

async fn scan(State(app): State<Arc<AppState>>, parent: String) -> RespResult<usize> {
    let plugins = app.pm().scan();

    let parent = UrlPath::new(&parent);
    let plugin = Plugin::new(&parent);

    for ele in &plugins {
        let plugin_id = plugin.path().new_path_with(ele.as_str());
        info_router(&plugin_id);
    }

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
