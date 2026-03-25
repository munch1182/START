pub use async_trait::async_trait;
pub use plugin_macro::bridge;
pub use serde_json::{Value, from_value, to_value};

pub type PluginResult<T, E = Box<dyn std::error::Error + Send + Sync>> = Result<T, E>;

pub mod prelude {
    pub use crate::{Plugin, PluginResult, Value, async_trait, bridge, from_value, to_value};
}

#[async_trait]
pub trait Plugin {
    async fn call(&self, input: Value) -> PluginResult<Value>;
}
