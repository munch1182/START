use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub libfile: String,
    pub uiurl: String,
}

impl PluginInfo {
    /// 通过父文件夹将相对文件路径转换为绝对路径
    pub fn canonicalize_by_parent(&self, parent: impl AsRef<std::path::Path>) -> Self {
        let parent = parent.as_ref();
        let libfile = parent.join(&self.libfile).to_string_lossy().to_string();
        let uiurl = if self.uiurl.starts_with("http://") || self.uiurl.starts_with("https://") {
            self.uiurl.clone()
        } else {
            parent.join(&self.uiurl).to_string_lossy().to_string()
        };
        Self {
            name: self.name.clone(),
            version: self.version.clone(),
            libfile,
            uiurl,
        }
    }
}
