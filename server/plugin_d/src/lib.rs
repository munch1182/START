mod resp;

use libcommon::{With, prelude::Result};
pub use resp::Resp;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::pin::Pin;

/// 插件处理函数名称
pub const NAME_HANLE: &[u8] = b"handle";
pub const NAME_GET_INO: &[u8] = b"get_info";

/// 返回的具体类型
///
/// [PluginResult]
pub type PluginResp = Result<Value>;
///
/// 返回的包装类型
///
/// ```ignore
/// #[unsafe(no_mangle)]
/// pub fn handle(path: String, req: Request<Body>) -> PluginResult {
///    Box::pin(async move { ... })
/// }
/// ```
pub type PluginResult = Pin<Box<dyn Future<Output = PluginResp> + Send>>;

#[derive(Debug, Clone, Serialize, Deserialize, With)]
pub struct PluginInfo {
    /// 插件名
    pub name: String,
    /// 插件版本
    pub version: String,
    /// 插件第二个关键字
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,
    /// 插件资源路径
    pub res: PluginRes,
}

#[derive(Debug, Clone, Serialize, Deserialize, With)]
pub struct PluginRes {
    pub file: String,
    pub html: String,
}

impl std::fmt::Display for &PluginInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.name, self.version)
    }
}

impl PluginInfo {
    pub fn new(
        name: impl ToString,
        version: impl ToString,
        file: impl ToString,
        html: impl ToString,
    ) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            keyword: None,
            res: PluginRes::new(file, html),
        }
    }

    pub fn default(name: impl ToString, version: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            keyword: None,
            res: PluginRes::default(),
        }
    }
}

impl PluginRes {
    pub fn new(file: impl ToString, html: impl ToString) -> Self {
        Self {
            file: file.to_string(),
            html: html.to_string(),
        }
    }
}

impl Default for PluginRes {
    fn default() -> Self {
        let file = {
            #[cfg(target_os = "windows")]
            {
                "index.dll"
            }
            #[cfg(target_os = "linux")]
            {
                "index.so"
            }
            #[cfg(target_os = "macos")]
            {
                "index.dylib"
            }
        };
        Self::new(file, "index.html")
    }
}
