use libcommon::prelude::info;
use plugin_manager::PluginManagerConfig;
use std::path::{Path, PathBuf};

pub const BASE_DIR_SCAN: &str = "test_dir_scan";
pub const BASE_DIR_SYSLINK: &str = "test_dir_fs";
pub const NAME_WEB_FILE: &str = "index";
pub const NAME_FILE_NET_PATH: &str = "fs";
pub const NUM_COUNT_CACHE: usize = 20;

#[derive(Debug)]
pub struct Config {
    /// 默认扫描文件夹
    ///
    /// 运行后会替换成真实地址
    /// 如果扫描插件时没提供具体地址，则默认扫描此地址
    pub scan_base_dir: String,
    /// 创建link的文件夹地址
    ///
    /// 运行后会替换成真实地址
    pub net_base_dir: String,
    /// 缓存数量
    pub num_cache: usize,
    /// link固定文件名
    pub web_file_name: String,
    /// 网络文件固定地址名
    pub net_path_name: String,
    /// 运行后的服务器完整地址
    pub host: Option<String>,
}

impl PluginManagerConfig for Config {
    fn symlink_base_dir(&self) -> PathBuf {
        PathBuf::from(&self.net_base_dir)
    }

    fn web_file_name(&self) -> String {
        self.web_file_name.to_string()
    }
}

impl Config {
    /// 设置文件夹
    ///
    ///
    /// 只能在开头设置, 不提供迁移
    pub fn setup_dir(&mut self, dir: impl AsRef<Path>, host: &str) {
        let dir = dir.as_ref();
        self.scan_base_dir = dir.join(BASE_DIR_SCAN).to_string_lossy().to_string();
        self.net_base_dir = dir.join(BASE_DIR_SYSLINK).to_string_lossy().to_string();
        self.host = Some(host.to_string());
        info!(
            "setup dir_scan: {:?}, dir_fs: {:?}",
            self.scan_base_dir, self.net_base_dir
        );
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scan_base_dir: BASE_DIR_SCAN.to_string(),
            net_base_dir: BASE_DIR_SYSLINK.to_string(),
            num_cache: NUM_COUNT_CACHE,
            web_file_name: NAME_WEB_FILE.to_string(),
            net_path_name: NAME_FILE_NET_PATH.to_string(),
            host: None,
        }
    }
}
