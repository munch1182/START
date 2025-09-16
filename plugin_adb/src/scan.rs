use libcommon::{newerr, prelude::Result};
use mdns_helper::{MdnsFinder, Packet, RData, rdata::Srv};
use std::{net::SocketAddr, time::Duration};

const QUERY: &str = "_adb-tls-connect._tcp.local";

pub fn scan_adb() -> Result<String> {
    let res: Vec<AdbSrv> = MdnsFinder::new()?
        .send_query(QUERY)?
        .recv_resp(Duration::from_secs(5), |r, s| (r, s).try_into().ok())?;
    res.first()
        .map(|r| format!("{}:{}", r.host, r.port))
        .ok_or(newerr!("no adb server found"))
}

#[derive(Debug, Clone)]
struct AdbSrv {
    pub name: String,
    pub host: String,
    pub port: u16,
}

impl AdbSrv {
    fn new(name: String, host: String, port: u16) -> Self {
        Self { name, host, port }
    }
}

impl TryInto<AdbSrv> for (SocketAddr, Packet<'_>) {
    type Error = libcommon::prelude::Err;

    fn try_into(self) -> std::result::Result<AdbSrv, Self::Error> {
        let host = self.0.ip().to_string();
        let data = self
            .1
            .additional
            .iter()
            .map(|r| {
                if let RData::SRV(Srv { port, .. }) = r.data {
                    let name = r.name.to_string().replace(QUERY, "");
                    Some(AdbSrv::new(name, host.clone(), port))
                } else {
                    None
                }
            })
            .filter(|r| r.is_some())
            .map(|a| a.unwrap())
            .collect::<Vec<_>>();
        data.iter()
            .max_by(|a, b| a.name.len().cmp(&b.name.len()))
            .cloned()
            .ok_or(newerr!("not find adb server"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libcommon::{log::log_setup, prelude::*};

    #[test]
    fn test_scan() -> Result<()> {
        log_setup();
        let res = scan_adb()?;
        info!("adb server: {}", res);
        Ok(())
    }
}
