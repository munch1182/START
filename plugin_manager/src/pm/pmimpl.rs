use crate::{
    config::{DIR_INDEX_NAME, NUM_CACHE, PLUGIN_HANDLE},
    pm::PluginId,
    router::APP_STATE,
    utils::file::scan_plugin,
};
use axum::{body::Body, http::Request};
use libcommon::{ext::FileDirCreateExt, newerr, prelude::*};
use libloading::{Library, Symbol};
use lru::LruCache;
use plugin_d::PluginInfo;
use serde_json::Value;
use std::{cell::RefCell, collections::HashMap, ffi::OsString, fs, num::NonZeroUsize, path::Path};

type PluginHandleFn = fn(String, Request<Body>) -> Result<Value>;

pub struct PM {
    scan_dir: OsString,
    plugins: RefCell<HashMap<PluginId, PluginHandle>>,
    handlers: RefCell<LruCache<PluginId, PluginHandleFn>>,
}

struct PluginHandle {
    pub(crate) info: PluginInfo,
    pub(crate) lib: Library,
}

impl PluginHandle {
    pub(crate) fn new(info: PluginInfo, lib: Library) -> Self {
        Self { info, lib }
    }
}

impl PM {
    pub fn new(scan_dir: impl Into<OsString>) -> Self {
        Self {
            scan_dir: scan_dir.into(),
            plugins: RefCell::new(HashMap::new()),
            handlers: RefCell::new(LruCache::new(NonZeroUsize::new(NUM_CACHE).unwrap())),
        }
    }

    pub fn info(&self) -> HashMap<PluginId, PluginInfo> {
        self.plugins
            .borrow()
            .iter()
            .map(|(k, v)| (k.clone(), v.info.clone()))
            .collect()
    }

    #[allow(unused)]
    pub fn update_dir(&mut self, new_dir: impl Into<OsString>) -> Vec<PluginId> {
        self.scan_dir = new_dir.into();
        self.scan()
    }

    pub fn scan(&self) -> Vec<PluginId> {
        info!("scan plugin dir: {:?}", self.scan_dir);
        let plugins = scan_plugin(&self.scan_dir);
        if plugins.is_empty() {
            return vec![];
        }
        let mut res = vec![];
        for plugin in plugins {
            if let Some(id) = self.load_plugin(plugin) {
                res.push(id);
            }
        }
        res
    }

    fn create_fs(&self, id: &PluginId, p: &PluginInfo) -> Result<()> {
        let res = &p.res;
        if res.html.is_empty() {
            return Ok(());
        }
        let dir = Path::new(APP_STATE.wait().fs_dir());
        let dir = dir.join(id.as_str()).create_dir()?;
        let index_in_fs = dir.join(DIR_INDEX_NAME);
        let index_in_orgin = res.html_with_dir();
        // 创建软链接，使用静态文件服务访问软连接间距访问文件
        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_file;
            symlink_file(&index_in_orgin, &index_in_fs)?;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink_file(&index_in_orgin, &index_in_fs)?;
        }
        Ok(())
    }

    pub fn remove(&self, id: &PluginId) -> bool {
        self.handlers.borrow_mut().pop(id);
        let remove = self.plugins.borrow_mut().remove(id).is_some();
        let dir = Path::new(APP_STATE.wait().fs_dir()).join(id.as_str());
        let _ = fs::remove_dir_all(&dir);
        if remove {
            warn!("remove plugin {id}");
        }
        remove
    }

    fn load_plugin(&self, p: PluginInfo) -> Option<PluginId> {
        let file = Path::new(&p.res.dir).join(&p.res.file);
        match unsafe { Library::new(&file) } {
            Ok(l) => {
                let id = PluginId::new_from(&p.name);
                self.remove(&id);
                let mut plugins = self.plugins.borrow_mut();
                let res = self.create_fs(&id, &p);
                info!(
                    "load plugin {}({id}) success, create fs: {}",
                    p.name,
                    res.is_ok()
                );
                plugins.insert(id.clone(), PluginHandle::new(p, l));
                Some(id)
            }
            Err(e) => {
                error!("load plugin {}({:#?}) failed: {e}", p.name, file);
                None
            }
        }
    }

    pub fn get(&self, id: PluginId) -> Option<PluginInfo> {
        self.plugins.borrow().get(&id).map(|p| p.info.clone())
    }

    pub fn handle(&self, id: &PluginId, path: String, req: Request<Body>) -> Result<Value> {
        let plugins = self.plugins.borrow();
        // 检查插件是否存在
        if !plugins.contains_key(id) {
            return Err(newerr!("Plugin not found: {}", id));
        }

        // 尝试从缓存中获取处理函数
        let mut handlers = self.handlers.borrow_mut();
        if let Some(cached_handler) = handlers.get(id) {
            return cached_handler(path, req);
        }

        // 缓存中没有，从插件库中获取处理函数
        let plugin = plugins.get(id).unwrap();
        // 获取 plugin_handle 符号
        let handle_fn: Symbol<PluginHandleFn> = unsafe { plugin.lib.get(PLUGIN_HANDLE) }?;

        // 将处理函数存入缓存
        handlers.put(id.clone(), *handle_fn);

        // 因为所有权，再从缓存中获取
        if let Some(cached_handler) = handlers.get(id) {
            return cached_handler(path, req);
        }
        Err(newerr!("Failed to get plugin_handle"))
    }
}
