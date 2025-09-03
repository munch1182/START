pub mod resp;

pub struct PluginInfo {
    pub name: String,
    pub version: String,
}

// pub type PlguinInfoExtren = extern "Rust" fn() -> PluginInfo;
// pub type PlguinHandler = extern "Rust" fn(Request<Body>) -> impl IntoResponse;
