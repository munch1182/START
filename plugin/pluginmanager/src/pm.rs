use crate::{PluginError, PluginInfo};
use dashmap::DashMap;
use libcommon::{New, hash};
use plugin::{Context, PluginResult};
use std::sync::Arc;

const NAME_PLUGIN_FN: &str = "plugin";
type PluginFn<'a> = libloading::Symbol<'a, unsafe fn() -> Box<dyn plugin::Plugin + Send + Sync>>;

#[derive(Default)]
pub struct PluginManager {
    plugins: DashMap<PluginId, Plugin>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PluginId(pub Arc<str>);

impl std::fmt::Display for PluginId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(New)]
struct Plugin {
    info: PluginInfo,
    load: LoadPlugin,
}

struct LoadPlugin {
    _lib: Arc<libloading::Library>,
    plugin: Arc<Box<dyn plugin::Plugin + Send + Sync>>,
}

impl PluginManager {
    pub fn load(&self, info: impl Into<PluginInfo>) -> Result<PluginId, PluginError> {
        let info = info.into();
        let load = LoadPlugin::load(&info.libfile)?;
        let id = PluginId::from(&info);

        let p = Plugin::new(info, load);
        self.plugins.insert(id.clone(), p);
        Ok(id)
    }

    pub fn unload(&self, id: &PluginId) {
        self.plugins.remove(id);
    }

    pub fn get(&self, id: &PluginId) -> Option<PluginInfo> {
        self.plugins.get(id).map(|p| p.info.clone())
    }

    pub fn list(&self) -> Vec<(PluginId, PluginInfo)> {
        self.plugins
            .iter()
            .map(|p| (p.key().clone(), p.info.clone()))
            .collect()
    }

    pub fn list_full_info(&self) -> Vec<(PluginId, PluginInfo)> {
        self.plugins
            .iter()
            .map(|p| (p.key().clone(), p.info.clone()))
            .collect()
    }

    pub async fn call(
        &self,
        id: &PluginId,
        arg: serde_json::Value,
        ctx: &dyn Context,
    ) -> PluginResult<serde_json::Value> {
        let p = self.plugins.get(id).ok_or(PluginError::PluginNotFound)?;
        p.load.plugin.call(arg, ctx).await
    }
}

impl LoadPlugin {
    pub(crate) fn load(path: impl AsRef<str>) -> Result<Self, PluginError> {
        let lib = unsafe { libloading::Library::new(path.as_ref()) }?;
        let plugin_fn = unsafe { lib.get::<PluginFn>(NAME_PLUGIN_FN.as_bytes()) }?;
        let plugin = unsafe { plugin_fn() };
        Ok(Self {
            _lib: Arc::new(lib),
            plugin: Arc::new(plugin),
        })
    }
}

impl From<&PluginInfo> for PluginId {
    fn from(value: &PluginInfo) -> Self {
        Self(Arc::from(hash!(value.name).to_string()))
    }
}
