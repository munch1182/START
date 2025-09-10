use serde::Serialize;

pub mod resp;

#[derive(Debug, Clone, Serialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
}

// pub type PlguinInfoExtren = extern "Rust" fn() -> PluginInfo;
// pub type PlguinHandler = extern "Rust" fn(Request<Body>) -> impl IntoResponse;

impl PluginInfo {
    #[inline]
    pub fn new(name: impl ToString, version: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
        }
    }
}
