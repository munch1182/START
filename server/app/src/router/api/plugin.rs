use crate::{CONFIG, config::DIR_DEFAULT_SCAN, pm::PluginManager};
use axum::{
    Json, Router,
    body::Body,
    extract::{Path, Request, State},
    routing::{any, get, post},
};
use plugin_d::Resp;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

pub fn routes<PM: PluginManagerState>() -> Router<PM> {
    Router::new()
        .route("/scan", post(scan::<PM>))
        .route("/list", get(list::<PM>))
        .route("/del", post(del::<PM>))
        .route("/{id}/{*query}", any(plugin::<PM>))
}

async fn plugin<PM: PluginManagerState>(
    State(app): State<PM>,
    Path((id, path)): Path<(String, String)>,
    req: Request<Body>,
) -> Resp<Value> {
    let res = get_pm(app).invoke(&id, path, req).await;
    res.into()
}

async fn scan<PM: PluginManagerState>(
    State(app): State<PM>,
    q: Option<Json<ScanQ>>,
) -> Resp<Vec<String>> {
    let query = q.map(|q| q.path.clone()).unwrap_or_else(|| {
        CONFIG
            .read()
            .map(|c| c.dir_fs.to_string())
            .unwrap_or(DIR_DEFAULT_SCAN.to_string())
    });
    get_pm(app).scan(query).into()
}

async fn list<PM: PluginManagerState>(State(app): State<PM>) -> Resp<Vec<ListR>> {
    get_pm(app)
        .get(|i| ListR {
            id: i.id.clone(),
            name: i.info.name.clone(),
            version: i.info.version.clone(),
        })
        .into()
}

async fn del<PM: PluginManagerState>(
    State(app): State<PM>,
    Json(query): Json<DelQ>,
) -> Resp<Option<String>> {
    get_pm(app).remove(&query.id).into()
}

pub trait PluginManagerState: Into<Arc<PluginManager>> + Clone + Send + Sync + 'static {}
impl<T> PluginManagerState for T where T: Into<Arc<PluginManager>> + Clone + Send + Sync + 'static {}

fn get_pm<PM: PluginManagerState>(app: PM) -> Arc<PluginManager> {
    app.into()
}

#[derive(Debug, Deserialize)]
struct ScanQ {
    path: String,
}

#[derive(Debug, Deserialize)]
struct DelQ {
    id: String,
}

#[derive(Debug, Serialize)]
struct ListR {
    id: String,
    name: String,
    version: String,
}
