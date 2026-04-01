use std::path::Path;

use host_pluginmanager::HostPM;
use libcommon::{debug, trace, warn};
use pluginmanager::{
    PluginError, PluginId,
    plugin::{self, Context, PluginResult, async_trait},
};
use walkdir::WalkDir;

use crate::AppState;

#[async_trait]
impl Context for AppState {
    fn log(&self, msg: &str) {
        debug!("{msg}")
    }
    async fn call_host(
        &self,
        cmd: &str,
        args: plugin::Value,
    ) -> plugin::PluginResult<plugin::Value> {
        if let Some(value) = host_pluginmanager::try_dispatch_host_p_m(self, cmd, args).await {
            return value;
        }
        Err("not implemented".into())
    }
}

fn map_err(e: PluginError) -> Box<dyn std::error::Error + Send + Sync> {
    format!("Plugin error: {}", e).into()
}

#[async_trait]
impl HostPM for AppState {
    async fn load_plugin(&self, arg: host_pluginmanager::PluginInfo) -> PluginResult<String> {
        let info = Into::<W<pluginmanager::PluginInfo>>::into(arg).0;
        let uiurl = info.uiurl.clone();
        let pid = self.pm.load(info).map_err(map_err)?;
        self.server.add_plugin_route(&pid.to_string(), uiurl);
        Ok(pid.to_string())
    }

    async fn unload_plugin(&self, arg: String) -> PluginResult<()> {
        let pid = PluginId(arg.into());
        self.pm.unload(&pid);
        self.server.remove_plugin_route(&pid.to_string());
        debug!("Unloaded plugin: {pid}");
        Ok(())
    }

    async fn reload_plugin(
        &self,
        arg: (String, host_pluginmanager::PluginInfo),
    ) -> PluginResult<()> {
        debug!("Reloading plugin: {}", &arg.0);
        self.unload_plugin(arg.0).await?;
        self.load_plugin(arg.1).await?;
        Ok(())
    }

    async fn list_plugins(
        &self,
        _: (),
    ) -> PluginResult<Vec<(String, host_pluginmanager::PluginInfo)>> {
        let list = self.pm.list_full_info();
        let result = list
            .into_iter()
            .map(|(pid, info)| {
                let info = W(info).into();
                (pid.to_string(), info)
            })
            .collect();
        Ok(result)
    }

    async fn scan(&self, arg: String) -> PluginResult<Vec<String>> {
        let dir = Path::new(&arg);
        if !dir.exists() {
            return Err(format!("directory does not exist: {arg}").into());
        }
        let mut result = Vec::new();
        for entry in WalkDir::new(dir)
            .max_depth(2)
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path();
            trace!("Scanning {path:?}");
            if path.is_file()
                && Some(mime_guess::mime::APPLICATION_JSON) == mime_guess::from_path(path).first()
            {
                match load_plugin_from_json(self, path).await {
                    Ok(id) => {
                        debug!("Loaded plugin from {path:?}: {id}");
                        result.push(id);
                    }
                    Err(e) => warn!("Failed to load plugin from {path:?}: {e}"),
                }
            }
        }
        Ok(result)
    }
}

async fn load_plugin_from_json(pm: &AppState, path: &Path) -> PluginResult<String> {
    trace!("try to loading plugin from {path:?}");
    let content = tokio::fs::read_to_string(path).await?;
    let info: pluginmanager::PluginInfo = serde_json::from_str(&content)?;
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let newinfo = info.canonicalize_by_parent(parent);
    trace!("{info:?} ==canonicalize==> {newinfo:?}");
    pm.load_plugin(W(newinfo).into()).await
}

impl From<host_pluginmanager::PluginInfo> for W<pluginmanager::PluginInfo> {
    fn from(value: host_pluginmanager::PluginInfo) -> Self {
        let info = pluginmanager::PluginInfo {
            name: value.name,
            version: value.version,
            libfile: value.libfile,
            uiurl: value.uiurl,
        };
        W(info)
    }
}

impl From<W<pluginmanager::PluginInfo>> for host_pluginmanager::PluginInfo {
    fn from(value: W<pluginmanager::PluginInfo>) -> Self {
        host_pluginmanager::PluginInfo {
            name: value.0.name,
            version: value.0.version,
            libfile: value.0.libfile,
            uiurl: value.0.uiurl,
        }
    }
}

struct W<T>(pub T);
