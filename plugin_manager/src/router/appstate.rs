use crate::{config::VERSION, pm::PM};
use serde_json::{Value, json};
use std::ffi::{OsStr, OsString};

pub struct AppState {
    config: AppConfig,
    version: &'static str,
    pm: PM,
}

unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}

#[derive(Debug, Clone)]
pub struct AppConfig {
    scan_dir: OsString,
    fs_dir: OsString,
}

impl AppState {
    pub(crate) fn new(config: AppConfig) -> Self {
        let version = VERSION;
        let pm = PM::new(config.scan_dir.as_os_str());
        Self {
            config,
            version,
            pm,
        }
    }

    pub(crate) fn pm(&self) -> &PM {
        &self.pm
    }

    pub(crate) fn scan_dir(&self) -> &OsStr {
        &self.config.scan_dir
    }

    pub(crate) fn fs_dir(&self) -> &OsStr {
        &self.config.fs_dir
    }

    pub(crate) fn version(&self) -> &str {
        self.version
    }

    pub(crate) fn config_str(&self) -> Value {
        json!({
            "scan_dir": self.scan_dir().to_string_lossy().to_string(),
            "fs_dir": self.fs_dir().to_string_lossy().to_string(),
            "version": self.version()
        })
    }
}

impl AppConfig {
    pub fn new(scan_dir: impl AsRef<OsStr>, fs_dir: impl AsRef<OsStr>) -> Self {
        Self {
            scan_dir: scan_dir.as_ref().to_os_string(),
            fs_dir: fs_dir.as_ref().to_os_string(),
        }
    }
}
