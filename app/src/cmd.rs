use crate::AppState;
use host_pluginmanager::Scan;
use libcommon::{Result, debug, warn};
use pluginmanager::PluginId;
use serde::{Deserialize, Serialize};
use window::{WindowState, bridge};

#[bridge]
pub async fn listplugins(WindowState(state): WindowState<AppState>) -> Result<Vec<PluginInfo>> {
    let list = state.pm.list_full_info();
    Ok(list.iter().map(From::from).collect())
}

#[bridge]
pub async fn scan(dir: String, WindowState(state): WindowState<AppState>) -> Vec<String> {
    match state.scan(dir).await {
        Ok(r) => r,
        Err(e) => {
            warn!("scan error: {e}");
            Vec::new()
        }
    }
}

#[bridge]
pub async fn callplugin(
    pluginid: String,
    method: String,
    params: serde_json::Value,
    pm: WindowState<AppState>,
) -> Result<serde_json::Value, String> {
    _call_plugin_method(pluginid, method, params, pm).await
}

async fn _call_plugin_method(
    pluginid: String,
    method: String,
    params: serde_json::Value,
    WindowState(state): WindowState<AppState>,
) -> Result<serde_json::Value, String> {
    let plugin_id = PluginId(pluginid.into());
    debug!("call plugin({plugin_id}) method: {method}, params: {params:?}");
    let input = serde_json::json!({ "method": method, "params": params});
    let result = state
        .pm
        .call(&plugin_id, input, state.as_ref())
        .await
        .map_err(|e| e.to_string())?;
    Ok(result)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PluginInfo {
    id: String,
    name: String,
    version: String,
    path: String,
}

impl From<&(PluginId, pluginmanager::PluginInfo)> for PluginInfo {
    fn from((id, info): &(PluginId, pluginmanager::PluginInfo)) -> Self {
        Self {
            id: id.0.to_string(),
            name: info.name.to_string(),
            version: info.version.to_string(),
            path: info.uiurl.to_string(),
        }
    }
}
