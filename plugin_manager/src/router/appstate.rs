use libcommon::{newerr, prelude::Result};
use serde_json::{Value, json};
use std::{
    ffi::{OsStr, OsString},
    sync::{Arc, OnceLock},
};

#[derive(Clone)]
pub struct AppState {
    config: AppConfig,
    version: &'static str,
}

const VERSION: &str = "0.0.1";

#[derive(Debug, Clone)]
pub struct AppConfig {
    scan_dir: OsString,
}

impl AppState {
    pub(crate) fn new(config: AppConfig) -> Self {
        let version = VERSION;
        Self { config, version }
    }

    pub(crate) fn scan_dir(&self) -> &OsStr {
        &self.config.scan_dir
    }

    pub(crate) fn version(&self) -> &str {
        self.version
    }

    pub(crate) fn config_str(&self) -> Value {
        json!({
            "scan_dir": self.scan_dir().to_string_lossy().to_string(),
            "version": self.version()
        })
    }
}

impl AppConfig {
    pub fn new(scan_dir: impl AsRef<OsStr>) -> Self {
        Self {
            scan_dir: scan_dir.as_ref().to_os_string(),
        }
    }
}

#[allow(dead_code)]
pub(crate) trait GetExt<'a, T> {
    fn call<U, F>(&'a self, f: F) -> Result<U>
    where
        F: FnOnce(T) -> Option<U>;

    fn call_if<U, F>(&'a self, f: F) -> Result<U>
    where
        F: FnOnce(T) -> Result<U>,
    {
        self.call(|a| f(a).ok())
    }
}

impl<'a> GetExt<'a, &'a AppState> for OnceLock<Arc<AppState>> {
    fn call<U, F>(&'a self, f: F) -> Result<U>
    where
        F: FnOnce(&'a AppState) -> Option<U>,
    {
        self.get()
            .and_then(|a| f(a))
            .ok_or(newerr!("err get APPSTATE"))
    }
}
