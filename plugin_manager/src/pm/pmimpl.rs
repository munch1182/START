use crate::{pm::PluginId, utils::file::scan_dir_find};
use axum::{body::Body, http::Request};
use libcommon::{
    newerr,
    prelude::{Result, info, warn},
};
use libloading::{Library, Symbol};
use lru::LruCache;
use plugin_d::PluginInfo;
use serde_json::Value;
use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    ffi::OsString,
    num::NonZeroUsize,
};

const PLUGIN_INFO: &[u8] = b"plugin_info";
const PLUGIN_HANDLE: &[u8] = b"plugin_handle";
const NUM_CACHE: usize = 20;

pub struct PM {
    scan_dir: OsString,
    libs: RefCell<HashMap<PluginId, Library>>,
    info: RefCell<HashMap<PluginId, PluginInfo>>,
    handler: RefCell<
        LruCache<
            PluginId,
            Symbol<'static, extern "Rust" fn(String, Request<Body>) -> Result<Value>>,
        >,
    >,
}

impl PM {
    pub fn new(scan_dir: impl Into<OsString>) -> Self {
        let scan_dir = scan_dir.into();
        let libs = RefCell::new(HashMap::new());
        let info = RefCell::new(HashMap::new());
        let handler = RefCell::new(LruCache::new(NonZeroUsize::new(NUM_CACHE).unwrap()));
        Self {
            scan_dir,
            libs,
            info,
            handler,
        }
    }

    #[allow(unused)]
    pub fn list(&self) -> Vec<PluginInfo> {
        self.info.borrow().values().cloned().collect()
    }

    pub fn info(&'_ self) -> Ref<'_, HashMap<PluginId, PluginInfo>> {
        self.info.borrow()
    }

    #[allow(unused)]
    pub fn update_dir(&mut self, new_dir: impl Into<OsString>) -> Vec<PluginId> {
        self.scan_dir = new_dir.into();
        self.scan()
    }

    pub fn scan(&self) -> Vec<PluginId> {
        let files = scan_dir_find(&self.scan_dir, |f| {
            f.extension().unwrap_or_default() == "dll"
        });
        if files.is_empty() {
            return vec![];
        }
        let mut res = vec![];
        for file in files {
            let lib = unsafe { Library::new(&file) };
            match lib {
                Err(e) => warn!("Failed to load {file:#?} plugin: {e}"),
                Ok(lib) => {
                    let info = unsafe { &lib.get::<extern "Rust" fn() -> PluginInfo>(PLUGIN_INFO) };
                    match info {
                        Err(e) => warn!("Failed to call plugin_info() from {file:#?}: {e}"),
                        Ok(i) => {
                            let info = i();
                            let id = self.add_info(info, lib);
                            info!("Loaded plugin {id:?} from {file:#?}");
                            res.push(id);
                        }
                    }
                }
            }
        }
        res
    }

    fn add_info(&self, info: PluginInfo, lib: Library) -> PluginId {
        let id = PluginId::new_from(&info);
        let mut libs = self.libs.borrow_mut();
        if libs.contains_key(&id) {
            self.remove(&id);
        }

        libs.insert(id.clone(), lib);
        let mut infos = self.info.borrow_mut();
        infos.insert(id.clone(), info);

        id
    }

    pub fn remove(&self, id: &PluginId) {
        self.info.borrow_mut().remove(id);
        self.libs.borrow_mut().remove(id);
        self.handler.borrow_mut().pop(id);
    }

    pub fn get(&self, id: PluginId) -> Option<PluginInfo> {
        self.info.borrow().get(&id).cloned()
    }

    pub fn handle(&self, id: PluginId, path: String, req: Request<Body>) -> Result<Value> {
        let lib = self.libs.borrow();
        let lib = lib.get(&id).ok_or(newerr!("Plugin {:?} not found", id))?;
        let mut handle = self.handler.borrow_mut();
        let handle = handle.get(&id).ok_or(libloading::Error::FreeLibraryUnknown);
        if let Ok(handle) = handle {
            return handle(path, req);
        }
        let handle = unsafe {
            lib.get::<extern "Rust" fn(String, Request<Body>) -> Result<Value>>(PLUGIN_HANDLE)
        };
        match handle {
            Err(e) => Err(e.into()),
            Ok(h) => h(path, req),
        }
    }
}
