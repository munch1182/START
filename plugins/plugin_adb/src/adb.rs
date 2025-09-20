use std::{net::SocketAddr, process::Command};

use libcommon::{newerr, prelude::Result};

/// 扫描局域网中可以连接的无线调试设备地址
///
/// 使用前需要关开无线调试以刷新
///
/// 方法1：使用adb mdns services命令
/// 方法2：使用mdns协议查询_adb-tls-connect._tcp.local
pub fn scan_adb() -> Result<Vec<String>> {
    let mut res = vec![];
    let mut cmd = Command::new("adb");
    cmd.args(["mdns", "services"]);
    let output = cmd.output()?;
    let lines = String::from_utf8(output.stdout)?;
    for line in lines.lines() {
        let addr = line.split_whitespace().last().unwrap_or("");
        if let Ok(addr) = addr.parse::<SocketAddr>() {
            res.push(addr.to_string());
        }
    }
    Ok(res)
}

pub fn connect_adb(addr: &str) -> Result<()> {
    let mut cmd = Command::new("adb");
    cmd.args(["connect", addr]);
    let state = cmd.status()?;
    if state.success() {
        return Ok(());
    }
    let output = cmd.output()?;
    let lines = String::from_utf8(output.stdout)?;
    Err(newerr!(lines))
}

pub fn disconnect_adb() -> Result<()> {
    let mut cmd = Command::new("adb");
    cmd.args(["disconnect"]);
    let state = cmd.status()?;
    if state.success() {
        return Ok(());
    }
    let output = cmd.output()?;
    let lines = String::from_utf8(output.stdout)?;
    Err(newerr!(lines))
}

#[cfg(test)]
mod tests {
    use super::*;
    use libcommon::{log::log_setup, prelude::info};

    #[test]
    fn test_scan() -> Result<()> {
        log_setup();
        let res = scan_adb()?;
        for ele in res {
            info!("adb server: {ele}");
        }
        Ok(())
    }
}
