mod pm;

use libcommon::hash;
use plugin_d::PluginInfo;
pub use pm::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PluginId(String);

impl From<&PluginInfo> for PluginId {
    fn from(value: &PluginInfo) -> Self {
        let id = format!("{:x}", hash!(value.name, value.version));
        Self(id)
    }
}
