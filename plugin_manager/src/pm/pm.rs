use crate::{pm::PluginId, utils::file::scan_dir_find};
use libcommon::prelude::{info, warn};
use libloading::Library;
use plugin_d::PluginInfo;
use std::{cell::RefCell, collections::HashMap, ffi::OsString};

pub struct PM {
    scan_dir: OsString,
    libs: RefCell<HashMap<PluginId, Library>>,
    info: RefCell<HashMap<PluginId, PluginInfo>>,
}

impl PM {
    pub fn new(scan_dir: impl Into<OsString>) -> Self {
        let scan_dir = scan_dir.into();
        let libs = RefCell::new(HashMap::new());
        let info = RefCell::new(HashMap::new());
        Self {
            scan_dir,
            libs,
            info,
        }
    }

    pub fn list(&self) -> Vec<PluginInfo> {
        self.info.borrow().values().cloned().collect()
    }

    #[allow(unused)]
    pub fn update_dir(&mut self, new_dir: impl Into<OsString>) -> usize {
        self.scan_dir = new_dir.into();
        self.scan()
    }

    pub fn scan(&self) -> usize {
        let files = scan_dir_find(&self.scan_dir, |f| {
            f.extension().unwrap_or_default() == "dll"
        });
        if files.is_empty() {
            return 0;
        }
        let mut update_count = 0;
        for file in files {
            let lib = unsafe { Library::new(&file) };
            match lib {
                Err(e) => warn!("Failed to load {file:#?} plugin: {e}"),
                Ok(lib) => {
                    let info =
                        unsafe { &lib.get::<extern "Rust" fn() -> PluginInfo>(b"plugin_info") };
                    match info {
                        Err(e) => warn!("Failed to call plugin_info() from {file:#?} plugin: {e}"),
                        Ok(i) => {
                            let plugin_info = i();
                            let id: PluginId = (&plugin_info).into();
                            let mut libs = self.libs.borrow_mut();
                            libs.insert(id.clone(), lib);
                            let mut info = self.info.borrow_mut();
                            info.insert(id.clone(), plugin_info);
                            info!("Loaded plugin {id:?} from {file:#?}");
                            update_count += 1;
                        }
                    }
                }
            }
        }
        update_count
    }
}
