pub use async_trait::async_trait;
pub use plugin_macro::call;
pub use serde_json::{Value, from_value, to_value};
pub type PluginResult<T, E = Box<dyn std::error::Error + Send + Sync>> = Result<T, E>;

pub mod prelude {
    pub use crate::{Plugin, PluginResult, Value, async_trait, call, from_value, to_value};
}

#[async_trait]
pub trait Context: Send + Sync {
    /// 插件将日志输出到主应用(可以利用主应用的日志文件写入)
    fn log(&self, _msg: &str) {}
    // 让host可以自行拓展而需要更改此处定义
    async fn call_host(&self, cmd: &str, _args: Value) -> PluginResult<Value> {
        Err(format!("Host command '{}' not supported now", cmd).into())
    }
}

#[async_trait]
pub trait Plugin {
    async fn call(&self, input: Value, ctx: &dyn Context) -> PluginResult<Value>;
}

impl Context for () {}
