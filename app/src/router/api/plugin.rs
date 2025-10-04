use crate::{config::Config, utils::opt::OptParam};
use axum::{
    Json, Router,
    body::Body,
    extract::{Path, Request, State},
    routing::{any, get, post},
};
use libcommon::newerr;
use plugin_d::Resp;
use plugin_manager::PluginManager;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

pub fn routes<PM: PluginManagerState>() -> Router<PM> {
    Router::new()
        .route("/scan", post(scan::<PM>))
        .route("/list", get(list::<PM>))
        .route("/del", post(del::<PM>))
        .route("/{id}", any(plugin_info::<PM>))
        .route("/{id}/{*query}", any(plugin::<PM>))
}
async fn plugin_info<PM: PluginManagerState>(
    State(app): State<PM>,
    Path(id): Path<String>,
) -> Resp<PluginInfoR> {
    let res = get_pm(app).find(&id, |i| PluginInfoR {
        id: i.info().id.clone(),
        name: i.info().info.name.clone(),
        keyword: i.info().info.keyword.clone(),
        version: i.info().info.version.clone(),
    });
    res.ok_or(newerr!("plugin not found: {id}")).into()
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
    OptParam(q): OptParam<ScanQ>,
) -> Resp<Vec<String>> {
    let pm = get_pm(app);
    let query = q
        .map(|q| q.path.clone())
        .unwrap_or_else(|| pm.config.scan_base_dir.clone());
    pm.scan(query).into()
}

async fn list<PM: PluginManagerState>(State(app): State<PM>) -> Resp<Vec<ListR>> {
    get_pm(app)
        .into_iter()
        .map(|i| ListR {
            id: i.id.clone(),
            name: i.info.name.clone(),
            keyword: i.info.keyword.clone(),
            version: i.info.version.clone(),
        })
        .collect::<Vec<_>>()
        .into()
}

async fn del<PM: PluginManagerState>(
    State(app): State<PM>,
    Json(query): Json<DelQ>,
) -> Resp<Option<String>> {
    get_pm(app).remove(&query.id).into()
}

pub trait PluginManagerState:
    Into<Arc<PluginManager<Config>>> + Clone + Send + Sync + 'static
{
}
impl<T> PluginManagerState for T where
    T: Into<Arc<PluginManager<Config>>> + Clone + Send + Sync + 'static
{
}

fn get_pm<PM: PluginManagerState>(app: PM) -> Arc<PluginManager<Config>> {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    keyword: Option<String>,
    version: String,
}

#[derive(Debug, Serialize)]
struct PluginInfoR {
    id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    keyword: Option<String>,
    version: String,
}
