mod scan;
use axum::{body::Body, extract::Query, http::Request};
use libcommon::{newerr, prelude::Result};
use plugin_d::{PluginInfo, Res};
use scan::*;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Mutex;

static ADB_CACHE: Mutex<Vec<String>> = Mutex::new(Vec::new());

#[unsafe(no_mangle)]
fn plugin_info(res: Res) -> PluginInfo {
    PluginInfo {
        name: "adb".to_string(),
        version: "0.0.1".to_string(),
        keyword: None,
        res,
    }
}

#[unsafe(no_mangle)]
fn plugin_handle(path: String, req: Request<Body>) -> Result<Value> {
    match F::from_str(path.as_str())? {
        F::Connect => connect(Query::try_from_uri(req.uri()).unwrap_or_default())?.to(),
        F::Disconnect => disconnect().to(),
        F::Scan => scan()?.to(),
    }
}

fn scan() -> Result<Resp<Vec<String>>> {
    let adb = scan_adb()?;
    {
        let mut cache = ADB_CACHE.lock().unwrap();
        cache.clear();
        cache.extend(adb.iter().cloned());
    }
    Ok(Resp::from_vec("scan", adb))
}

fn disconnect() -> Resp<String> {
    let res = disconnect_adb();
    {
        ADB_CACHE.lock().unwrap().clear();
    }
    Resp::from("disconnect", res)
}

fn connect(Query(Connect(i)): Query<Connect>) -> Result<Resp<String>> {
    let adb = {
        let cache = ADB_CACHE.lock().unwrap();
        let cache = cache.get(i as usize);
        cache.cloned()
    };
    if let Some(adb) = adb {
        let res = connect_adb(&adb);
        return Ok(Resp::from("connect", res));
    }
    Ok(Resp::from("connect", Err(newerr!("no adb device"))))
}

#[derive(Default, Deserialize)]
struct Connect(pub u8);

#[derive(serde::Serialize)]
struct Resp<T> {
    pub op: String,
    pub res: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: serde::Serialize> Resp<T> {
    fn to(self) -> Result<Value> {
        Ok(serde_json::to_value(self)?)
    }
}

impl Resp<Vec<String>> {
    fn from_vec(op: impl ToString, res: Vec<String>) -> Self {
        Self {
            op: op.to_string(),
            res: true,
            data: Some(res),
        }
    }
}

impl Resp<String> {
    fn from(op: impl ToString, res: Result<()>) -> Self {
        Self {
            op: op.to_string(),
            res: res.is_ok(),
            data: res.err().map(|e| Some(e.to_string())).unwrap_or_default(),
        }
    }
}

#[derive(Debug)]
enum F {
    Scan,
    Connect,
    Disconnect,
}

impl F {
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "connect" => Ok(Self::Connect),
            "disconnect" => Ok(Self::Disconnect),
            "scan" => Ok(Self::Scan),
            _ => Err(newerr!("unknown fun: {s}")),
        }
    }
}
