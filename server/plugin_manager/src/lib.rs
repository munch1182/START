mod scan;

use crate::scan::scan_info;
use axum::{body::Body, http::Request};
use libcommon::{
    Default_With, With,
    ext::FileDirCreateExt,
    hash, newerr,
    prelude::{ErrMapperExt, Result, debug, info, warn},
};
use libloading::{Library, Symbol};
use lru::LruCache;
use parking_lot::{Mutex, RwLock};
use plugin_d::{NAME_HANLE, PluginInfo, PluginRes, PluginResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs,
    num::NonZero,
    path::{Path, PathBuf},
    sync::Arc,
};

pub type PluginHandleFn = fn(String, Request<Body>) -> PluginResult;
type PluginId = String;
type OnUpdate<CONFIG> = Box<dyn Fn(&PluginManager<CONFIG>)>;

/// 插件路径相关配置
pub trait PluginManagerConfig {
    /// 创建的链接所处的目录
    fn symlink_base_dir(&self) -> PathBuf;
    /// 创建的链接的名称
    fn web_file_name(&self) -> String;
    /// 缓存方法数量
    fn num_cache(&self) -> usize {
        20
    }
}

/// 插件管理器
///
/// 管理插件加载、卸载、调用等功能
pub struct PluginManager<CONFIG: PluginManagerConfig> {
    pub(crate) plugins: Arc<RwLock<HashMap<PluginId, PluginHandle>>>,
    pub(crate) fn_caches: Arc<Mutex<LruCache<PluginId, PluginHandleFn>>>,
    pub config: Arc<CONFIG>,
    pub(crate) on_update: Arc<RwLock<Option<OnUpdate<CONFIG>>>>,
}

unsafe impl<CONFIG: PluginManagerConfig> Send for PluginManager<CONFIG> {}
unsafe impl<CONFIG: PluginManagerConfig> Sync for PluginManager<CONFIG> {}

impl<CONFIG: PluginManagerConfig + Default> Default for PluginManager<CONFIG> {
    fn default() -> Self {
        let config = CONFIG::default();
        let num_cache = config.num_cache();
        Self {
            plugins: Default::default(),
            fn_caches: Arc::new(Mutex::new(LruCache::new(NonZero::new(num_cache).unwrap()))),
            config: Arc::new(config),
            on_update: Default::default(),
        }
    }
}

/// 插件对象
///
/// 持有插件的动态链接库和插件信息
pub struct PluginHandle {
    pub(crate) info: PluginInfoWrapper,
    pub(crate) lib: Library,
}

/// 插件信息
///
/// 包含插件的基本信息，以及插件在服务器中的信息
#[derive(Debug, Clone, Serialize, Deserialize, With, Default_With)]
pub struct PluginInfoWrapper {
    #[default_with(no_default)]
    pub id: PluginId,
    #[default_with(no_default)]
    pub info: PluginInfo,
    pub info_net: Option<PluginRes>,
}

impl<CONFIG: PluginManagerConfig + Send + Sync> PluginManager<CONFIG> {
    pub fn new(config: Arc<CONFIG>) -> Self {
        Self {
            plugins: Default::default(),
            fn_caches: Arc::new(Mutex::new(LruCache::new(
                NonZero::new(config.num_cache()).unwrap(),
            ))),
            config,
            on_update: Default::default(),
        }
    }

    /// 扫描目标文件夹并加载其中的所有有效插件
    ///
    /// 目前会扫描其中有效的json文件并以此构建插件；
    /// 如果该插件已经存在，则会替换该插件信息
    pub fn scan(&self, dir: impl AsRef<Path>) -> Result<Vec<PluginId>> {
        let mut res = vec![];
        let dir = dir.as_ref();
        let infos = scan_info(dir)?;
        debug!("Scan {dir:?}: Found {} plugins", infos.len());
        for i in infos {
            match self.load_plugin(i) {
                Ok(id) => {
                    info!("Plugin {id} loaded");
                    res.push(id);
                }
                Err(e) => warn!("Plugin {dir:?} load failed: {e}"),
            }
        }
        self.call_update();
        Ok(res)
    }

    pub fn set_on_update(self, f: impl Fn(&Self) + 'static) -> Self {
        *self.on_update.write() = Some(Box::new(f));
        self
    }

    fn call_update(&self) {
        if let Some(f) = self.on_update.write().as_ref() {
            f(self);
        }
    }

    /// 查找插件并返回转换后的信息
    pub fn find<MAP, R>(&self, id: &PluginId, map: MAP) -> Option<R>
    where
        MAP: Fn(&PluginHandle) -> R,
    {
        self.plugins.read().get(id).map(map)
    }

    /// 移除并卸载插件
    pub fn remove(&self, id: &PluginId) -> Option<String> {
        self.fn_caches.lock().pop(id);
        if let Some(plugin) = self.plugins.write().remove(id) {
            info!("Plugin {id} removed");
            let name = plugin.info.info.name.clone();
            drop(plugin);
            self.call_update();
            return Some(name);
        }
        None
    }

    /// 调用插件来处理请求
    ///
    /// 需要尽快释放锁
    pub async fn invoke(&self, id: &PluginId, path: String, req: Request<Body>) -> Result<Value> {
        {
            if !self.plugins.read().contains_key(id) {
                return Err(newerr!("Plugin {id} not found"));
            }
        }
        self.get_or_load_handler(id).await?(path, req).await
    }

    async fn get_or_load_handler(&self, id: &PluginId) -> Result<PluginHandleFn> {
        // 先检查缓存
        {
            if let Some(handler) = self.fn_caches.lock().get(id).copied() {
                return Ok(handler);
            }
        }

        // 缓存中没有，加载并缓存
        let handler = {
            let plugins = self.plugins.read();
            let plugin = plugins.get(id).newerr()?;
            unsafe { plugin.lib.get(NAME_HANLE) }
                .map(|sym: Symbol<PluginHandleFn>| *sym)
                .map_err(|e| newerr!(e))?
        };

        // 放入缓存
        {
            self.fn_caches.lock().put(id.clone(), handler);
        }

        Ok(handler)
    }

    fn load_plugin(&self, (dir, i): (PathBuf, PluginInfo)) -> Result<PluginId> {
        let id = generate_id(&i.name);
        let info_net = self.generate_res_net(&id, dir, &i)?;
        match unsafe { Library::new(&info_net.file) } {
            Ok(library) => {
                let info = PluginInfoWrapper::default_with(id.clone(), i).with_info_net(info_net);
                let (k, v) = (id.clone(), PluginHandle::new(info, library));
                {
                    self.plugins.write().insert(k, v);
                }
                Ok(id)
            }
            Err(e) => Err(newerr!(e)),
        }
    }

    /// 根据现有的插件信息，生成相关网络资源并将其返回
    ///
    /// 相关操作:
    /// 1. 补充并规范插件文件地址
    /// 2. 创建HTML文件的软链接以便静态文件服务访问
    fn generate_res_net(
        &self,
        id: &str,
        parent: PathBuf,
        plugin_info: &PluginInfo,
    ) -> Result<PluginRes> {
        let res = &plugin_info.res;
        // 规范化文件路径
        let (library_path, html_path) = self.normalize_plugin_paths(&parent, res)?;
        // 创建软链接
        let html_link_path = self.create_html_symlink(id, &html_path)?;

        let plugin_res = PluginRes {
            html: html_link_path.to_string_lossy().to_string(),
            file: library_path.to_string_lossy().to_string(),
        };
        info!("Plugin {id} network resources generated: {plugin_res:?}");
        Ok(plugin_res)
    }

    /// 规范化插件文件路径
    fn normalize_plugin_paths(&self, parent: &Path, res: &PluginRes) -> Result<(PathBuf, PathBuf)> {
        let normalize_path = |path_str: &str| -> Result<PathBuf> {
            let path = Path::new(path_str);
            // 验证路径存在且不为文件
            if path.exists() && !path.is_file() {
                return Err(newerr!("Path is not a file: {path_str}"));
            }
            // 处理相对/绝对路径
            Ok(if path.is_relative() {
                parent.join(path)
            } else {
                path.to_path_buf()
            })
        };

        let library_path = normalize_path(&res.file)?;
        let html_path = normalize_path(&res.html)?;

        Ok((library_path, html_path))
    }

    /// 为HTML文件创建软链接
    fn create_html_symlink(&self, plugin_id: &str, html_path: &Path) -> Result<PathBuf> {
        let symlink_path = self
            .config
            .symlink_base_dir()
            .join(plugin_id)
            .join(self.config.web_file_name())
            .create_parent()?;

        self.create_or_replace_symlink(html_path, &symlink_path)?;
        info!("Created symlink for plugin {plugin_id}: {html_path:?} ==> {symlink_path:?}");
        Ok(symlink_path)
    }

    /// 创建或替换软链接（跨平台）
    fn create_or_replace_symlink(&self, target: &Path, link_path: &Path) -> Result<()> {
        debug!("Creating or replacing symlink: {target:?} -> {link_path:?}");
        // 清理已存在的链接
        if let Some(dir) = link_path.parent() {
            // 不能判断link是否存在
            fs::remove_dir_all(dir)
                .map_err(|e| newerr!("Failed to remove existing symlink: {e}"))?;
            fs::create_dir_all(dir)
                .map_err(|e| newerr!("Failed to create symlink directory: {e}"))?;
        }

        // 平台特定的软链接创建
        self.create_symlink_platform(target, link_path)
            .map_err(|e| newerr!("Failed to create symlink: {e}"))
    }

    /// 平台特定的软链接实现
    fn create_symlink_platform(&self, target: &Path, link_path: &Path) -> std::io::Result<()> {
        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_file;
            symlink_file(target, link_path)
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(target, link_path)
        }

        #[cfg(not(any(windows, unix)))]
        {
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Symbolic links not supported on this platform",
            ))
        }
    }
}

impl PluginHandle {
    pub(crate) fn new(info: PluginInfoWrapper, lib: Library) -> Self {
        Self { info, lib }
    }

    pub fn info(&self) -> &PluginInfoWrapper {
        &self.info
    }
}

pub(crate) fn generate_id(name: &str) -> PluginId {
    format!("{:x}", hash!(name))
}

impl std::fmt::Display for PluginInfoWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.info.name, self.info.version)
    }
}

impl<CONFIG: PluginManagerConfig> IntoIterator for &PluginManager<CONFIG> {
    type Item = PluginInfoWrapper;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.plugins
            .read()
            .values()
            .map(|p| p.info.clone())
            .collect::<Vec<_>>()
            .into_iter()
    }
}
