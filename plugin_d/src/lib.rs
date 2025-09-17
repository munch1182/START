use libcommon::prelude::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub mod resp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// 插件名
    pub name: String,
    /// 插件版本
    pub version: String,
    /// 插件第二个关键字
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,
    /// 插件资源
    pub res: Res,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Res {
    /// 插件资源文件夹
    pub dir: String,
    /// 插件dll文件名(文件夹内)
    pub file: String,
    /// 插件html文件名(文件夹内)，
    pub html: String,
}

impl Res {
    pub fn file_with_dir(&self) -> PathBuf {
        Path::new(&self.dir).join(&self.file)
    }
    pub fn html_with_dir(&self) -> PathBuf {
        Path::new(&self.dir).join(&self.html)
    }
}

impl PluginInfo {
    #[inline]
    pub fn from_json(path: impl AsRef<Path>) -> Result<Self> {
        Ok(serde_json::from_reader(fs::File::open(path)?)?)
    }
}

impl Res {
    ///
    /// 创建插件资源
    /// dll文件使用默认名：plugin_{plugin_name}.dll
    /// html文件使用默认名：index.html
    pub fn new_in_dir(plugin_name: &str) -> Self {
        let file = if plugin_name.starts_with("plugin_") {
            format!("{plugin_name}.dll")
        } else {
            format!("plugin_{plugin_name}.dll")
        };
        let html = String::from("index.html");
        let dir = String::default();
        Self { dir, file, html }
    }
}

impl std::fmt::Display for &PluginInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
