use context::define_host_group;
// host-api/src/plugin_mgmt.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub libfile: String,
    pub uiurl: String,
}

type Pid = String;

define_host_group! {
    HostPM,
    /// 加载插件
    (load_plugin, PluginInfo, Pid),
    /// 卸载插件
    (unload_plugin, Pid, ()),
    /// 重新加载插件
    (reload_plugin, (Pid, PluginInfo), ()),
    /// 获取所有已加载的插件
    (list_plugins, (), Vec<(Pid, PluginInfo)>),
    /// 扫描指定文件夹，根据其中2级文件夹内的*.json解析成PluginInfo格式
    (scan, String, Vec<Pid>),
}
