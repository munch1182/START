mod scan;

use crate::{CONFIG, config::NUM_COUNT_CACHE, pm::scan::scan_info};
use axum::{body::Body, http::Request};
use libcommon::{
    Default_With, With,
    ext::FileDirCreateExt,
    hash, newerr,
    prelude::{ErrMapperExt, Result, info, warn},
};
use libloading::{Library, Symbol};
use lru::LruCache;
use plugin_d::{NAME_HANLE, PluginInfo, PluginRes, PluginResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs,
    num::NonZero,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
};

pub type PluginHandleFn = fn(String, Request<Body>) -> PluginResult;
type PluginId = String;

pub struct PluginManager {
    pub(crate) plugins: Arc<RwLock<HashMap<PluginId, PluginHandle>>>,
    pub(crate) fn_caches: Arc<Mutex<LruCache<PluginId, PluginHandleFn>>>,
}

unsafe impl Send for PluginManager {}
unsafe impl Sync for PluginManager {}

impl Default for PluginManager {
    fn default() -> Self {
        let num = CONFIG
            .read()
            .map(|c| c.num_cache)
            .unwrap_or(NUM_COUNT_CACHE);
        Self {
            plugins: Default::default(),
            fn_caches: Arc::new(Mutex::new(LruCache::new(NonZero::new(num).unwrap()))),
        }
    }
}

pub struct PluginHandle {
    pub(crate) info: PluginInfoWrapper,
    pub(crate) lib: Library,
}

#[derive(Debug, Clone, Serialize, Deserialize, With, Default_With)]
pub struct PluginInfoWrapper {
    #[default_with(no_default)]
    pub(crate) id: PluginId,
    #[default_with(no_default)]
    pub(crate) info: PluginInfo,
    pub(crate) info_net: Option<PluginRes>,
}

impl PluginManager {
    pub fn scan(&self, dir: impl AsRef<Path>) -> Result<Vec<PluginId>> {
        let mut res = vec![];
        let dir = dir.as_ref();
        let infos = scan_info(dir)?;
        info!("Scan {:?}: Found {} plugins", dir, infos.len());
        for i in infos {
            match self.load_plugin(i) {
                Ok(id) => {
                    info!("Plugin {id} loaded");
                    res.push(id);
                }
                Err(e) => warn!("Plugin {dir:?} load failed: {e}"),
            }
        }
        Ok(res)
    }

    pub fn get<MAP, R>(&self, map: MAP) -> Vec<R>
    where
        MAP: Fn(&PluginInfoWrapper) -> R,
    {
        self.plugins
            .read()
            .unwrap()
            .values()
            .map(|i| map(&i.info))
            .collect::<Vec<_>>()
    }
    pub fn find<MAP, R>(&self, id: &PluginId, map: MAP) -> Option<R>
    where
        MAP: Fn(&PluginHandle) -> R,
    {
        self.plugins.read().unwrap().get(id).map(map)
    }

    pub fn remove(&self, id: &PluginId) -> Option<String> {
        info!("Plugin {id} removed");
        self.fn_caches.lock().ok()?.pop(id);
        self.plugins
            .write()
            .unwrap()
            .remove(id)
            .map(|i| i.info.info.name)
    }

    /// 调用插件来处理请求
    ///
    /// 需要尽快释放锁
    pub async fn invoke(&self, id: &PluginId, path: String, req: Request<Body>) -> Result<Value> {
        {
            let plugins = self.plugins.read().newerr()?;
            if !plugins.contains_key(id) {
                return Err(newerr!("Plugin {id} not found"));
            }
        }
        self.get_or_load_handler(id).await?(path, req).await
    }

    async fn get_or_load_handler(&self, id: &PluginId) -> Result<PluginHandleFn> {
        // 先检查缓存
        {
            let mut caches = self.fn_caches.lock().newerr()?;
            if let Some(handler) = caches.get(id).copied() {
                return Ok(handler);
            }
        }

        // 缓存中没有，加载并缓存
        let handler = {
            let plugins = self.plugins.read().newerr()?;
            let plugin = plugins.get(id).unwrap();
            unsafe { plugin.lib.get(NAME_HANLE) }
                .map(|sym: Symbol<PluginHandleFn>| *sym)
                .map_err(|e| newerr!(e))?
        };

        // 放入缓存
        {
            let mut caches = self.fn_caches.lock().newerr()?;
            caches.put(id.clone(), handler);
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
                    self.plugins.write().newerr()?.insert(k, v);
                }
                Ok(id)
            }
            Err(e) => Err(newerr!(e)),
        }
    }

    fn generate_res_net(&self, id: &str, parent: PathBuf, i: &PluginInfo) -> Result<PluginRes> {
        let dir_fs = { CONFIG.read().newerr()?.dir_fs.clone() };
        let res = &i.res;
        let map_file = |str: &str| {
            let path = Path::new(str);
            if path.exists() && !path.is_file() {
                Err(newerr!("File error: {str} is not a file"))
            } else if path.is_relative() {
                Ok(parent.join(path))
            } else {
                Ok(path.to_path_buf())
            }
        };
        let file = &map_file(&res.file)?;
        let html = &map_file(&res.html)?;
        let link = &Path::new(&dir_fs)
            .join(CONFIG.read().newerr()?.get_fs_path(id))
            .create_parent()?;
        info!("Generate link: {id}: {html:?} -> {link:?}");
        if link.exists() {
            let _ = fs::remove_file(link);
        }
        // 创建软链接，使用静态文件服务访问软连接间距访问文件
        // window可以打开开发者模式避开权限要求
        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_file;
            symlink_file(html, link)?;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink_file(html, link)?;
        }
        let html = link.to_string_lossy().to_string();
        let file = file.to_string_lossy().to_string();
        let res = PluginRes { html, file };
        info!("Plugin {id} res_net: {res:?}");
        Ok(res)
    }
}

impl PluginHandle {
    pub(crate) fn new(info: PluginInfoWrapper, lib: Library) -> Self {
        Self { info, lib }
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

#[cfg(test)]
mod tests {
    use super::*;
    use libcommon::log_setup;
    use std::env::current_dir;

    #[ignore = "need files"]
    #[test]
    fn test_scan() -> Result<()> {
        log_setup();
        let dir = current_dir()?;
        let dir = dir.parent().unwrap().parent().unwrap();
        {
            CONFIG.write().newerr()?.setup_dir(&dir);
        }
        let pm = PluginManager::default();
        pm.scan(CONFIG.read().newerr()?.dir_scan.clone())?;
        Ok(())
    }
}
