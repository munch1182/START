mod scan;
pub use scan::*;

use axum::{body::Body, http::Request};
use libcommon::prelude::Result;
use serde_json::{Value, json};
use std::env::current_dir;

#[unsafe(no_mangle)]
fn plugin_handle(path: String, _req: Request<Body>) -> Result<Value> {
    let curr = current_dir()?;
    let curr = curr.to_str().unwrap_or_default();
    Ok(json!({
        "path": path,
        "curr": curr,
    }))
}

#[cfg(test)]
mod tests {
    use libcommon::{
        curr_dir,
        ext::{FileDirCreateExt, PathJoinExt, PrettyStringExt},
        log::log_setup,
        prelude::{Result, info, timer},
    };
    use plugin_d::PluginInfo;
    use std::{env::current_dir, fs, process::Command};

    #[timer]
    #[test]
    fn generate_dll() -> Result<()> {
        log_setup();
        let generate_dll = curr_dir!("target", "release", "plugin_adb.dll")?;
        let scan_dir_parent = current_dir()?.parent().unwrap().join_all(&[
            "plugin_manager",
            "test_scan_dir",
            "plugin_adb",
        ]);

        let scan_dir_dll = scan_dir_parent.join("plugin_adb.dll");
        let scan_dir_html = scan_dir_parent.join("index.html");
        let scan_dir_json = scan_dir_parent.join("a.json");
        if scan_dir_parent.exists() {
            std::fs::remove_dir_all(&scan_dir_parent)?;
        }
        scan_dir_parent.create_dir()?;
        let mut cmd = Command::new("cargo");
        cmd.args(&["build", "--release"]);
        let output = cmd.output()?;
        info!("execute: {} : {}", cmd.to_string_pretty(), output.status);

        assert!(generate_dll.exists());
        info!("target: {generate_dll:?}");

        fs::copy(generate_dll, scan_dir_dll)?;
        fs::write(
            scan_dir_html,
            r#"<!DOCTYPE html>
<html>
    <body>
        <h1>Hello, world!</h1>
    </body>
</html>"#,
        )?;
        let info = PluginInfo::new_in_dir_default("adb", "0.1.0");
        let str = serde_json::to_string_pretty(&info)?;
        fs::write(scan_dir_json, str)?;
        Ok(())
    }
}
