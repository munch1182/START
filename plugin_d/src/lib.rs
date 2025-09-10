use serde::Serialize;

pub mod resp;

#[derive(Debug, Clone, Serialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
}

impl PluginInfo {
    #[inline]
    pub fn new(name: impl ToString, version: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
        }
    }
}

impl ToString for &PluginInfo {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}
