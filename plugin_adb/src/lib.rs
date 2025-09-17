mod scan;
use axum::{body::Body, extract::Query, http::Request};
use libcommon::{newerr, prelude::Result};
use scan::*;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Mutex;

static ADB_CACHE: Mutex<Vec<String>> = Mutex::new(Vec::new());

#[unsafe(no_mangle)]
fn plugin_handle(path: String, req: Request<Body>) -> Result<Value> {
    match F::from_str(path.as_str())? {
        F::Connect => {
            let i: Query<Connect> = Query::try_from_uri(req.uri()).unwrap_or_default();
            let adb = {
                let cache = ADB_CACHE.lock().unwrap();
                let cache = cache.get(i.i as usize);
                cache.cloned()
            };
            if let Some(adb) = adb {
                let res = connect_adb(&adb);
                return Resp::from("connect", res).to();
            }
            Resp::from("connect", Err(newerr!("no adb device"))).to()
        }
        F::Disconnect => {
            let res = disconnect_adb();
            {
                ADB_CACHE.lock().unwrap().clear();
            }
            Resp::from("connect", res).to()
        }
        F::Scan => {
            let adb = scan_adb()?;
            {
                let mut cache = ADB_CACHE.lock().unwrap();
                cache.clear();
                cache.extend(adb.iter().cloned());
            }
            Resp::from_vec("connect", adb).to()
        }
    }
}

#[derive(Default, Deserialize)]
struct Connect {
    pub i: u8,
}

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
