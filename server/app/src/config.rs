use std::{
    path::{Path, PathBuf},
    sync::{Arc, LazyLock, RwLock},
};

use libcommon::prelude::info;

pub const DIR_DEFAULT_SCAN: &str = "test_dir_scan";
pub const DIR_DEFAULT_FS: &str = "test_dir_fs";
pub const NAME_FILE_LINK: &str = "index";
pub const NAME_FILE_NET: &str = "fs";
pub const NUM_COUNT_CACHE: usize = 20;

pub static CONFIG: LazyLock<Arc<RwLock<Config>>> = LazyLock::new(Default::default);

#[derive(Debug)]
pub struct Config {
    /// 默认扫描文件夹
    ///
    /// 运行后会替换成真实地址
    pub dir_scan: String,
    /// 创建link的文件夹地址
    ///
    /// 运行后会替换成真实地址
    pub dir_fs: String,
    /// 缓存数量
    pub num_cache: usize,
    /// link固定文件名
    pub name_file_link: String,
    /// 网络文件固定地址名
    pub name_file_net: String,
    /// 运行后的服务器完整地址
    pub host: Option<String>,
}

impl Config {
    pub fn get_fs_path(&self, id: &str) -> PathBuf {
        Path::new(&self.dir_fs).join(id).join(&self.name_file_link)
    }

    /// 设置文件夹
    ///
    ///
    /// 只能在开头设置, 不提供迁移
    pub fn setup_dir(&mut self, dir: impl AsRef<Path>) {
        let dir = dir.as_ref();
        self.dir_scan = dir.join(DIR_DEFAULT_SCAN).to_string_lossy().to_string();
        self.dir_fs = dir.join(DIR_DEFAULT_FS).to_string_lossy().to_string();
        info!(
            "setup dir_scan: {:?}, dir_fs: {:?}",
            self.dir_scan, self.dir_fs
        );
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dir_scan: DIR_DEFAULT_SCAN.to_string(),
            dir_fs: DIR_DEFAULT_FS.to_string(),
            num_cache: NUM_COUNT_CACHE,
            name_file_link: NAME_FILE_LINK.to_string(),
            name_file_net: NAME_FILE_NET.to_string(),
            host: None,
        }
    }
}
