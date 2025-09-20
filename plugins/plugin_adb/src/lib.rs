use std::sync::Mutex;

use axum::{body::Body, http::Request};
use libcommon::{
    newerr,
    prelude::{ErrMapperExt, Result},
};
use plugin_d::{PluginInfo, PluginResult};
use serde::Deserialize;
use serde_json::{Value, json};

mod adb;

const NAME: &str = "adb";
const VERSION: &str = "0.0.1";

#[unsafe(no_mangle)]
pub fn get_info() -> PluginInfo {
    PluginInfo::default(NAME, VERSION)
}

static ADB_CACHE: Mutex<Vec<String>> = Mutex::new(Vec::new());

#[unsafe(no_mangle)]
pub fn handle(path: String, req: Request<Body>) -> PluginResult {
    Box::pin(async move {
        let action = F::from_request(&path, req).await?;
        let resp = action.handle()?;
        Ok(resp)
    })
}

enum F {
    Scan,
    Connect(usize),
    Disconnect,
}

impl F {
    async fn from_request(path: &str, req: Request<Body>) -> Result<Self> {
        match path {
            "scan" => Ok(F::Scan),
            "connect" => {
                let query: ConnectQuery = {
                    let bytes = axum::body::to_bytes(req.into_body(), 2_000_000).await?;
                    serde_json::from_slice(&bytes)
                }
                .unwrap_or_default();
                Ok(F::Connect(query.i))
            }
            "disconnect" => Ok(F::Disconnect),
            _ => Err(newerr!("Unknown path: {path}"))?,
        }
    }

    fn handle(&self) -> Result<Value> {
        match self {
            F::Scan => {
                let devices = adb::scan_adb()?;
                {
                    let mut cache = ADB_CACHE.lock().newerr()?;
                    cache.clear();
                    cache.extend(devices.iter().cloned());
                }
                Ok(json!({ "devs": devices }))
            }
            F::Connect(id) => {
                let addr = {
                    let cache = ADB_CACHE.lock().newerr()?;
                    cache.get(*id).cloned()
                }
                .ok_or(newerr!("Invalid device id: {id}"))?;
                adb::connect_adb(&addr)?;
                Ok(json!(true))
            }
            F::Disconnect => {
                {
                    let mut cache = ADB_CACHE.lock().newerr()?;
                    cache.clear();
                }
                adb::disconnect_adb()?;
                Ok(json!(true))
            }
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct ConnectQuery {
    i: usize,
}
