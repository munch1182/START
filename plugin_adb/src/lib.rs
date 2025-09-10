use plugin_d::PluginInfo;

#[unsafe(no_mangle)]
extern "Rust" fn plugin_info() -> PluginInfo {
    PluginInfo::new("adb", "0.1.0")
}

#[cfg(test)]
mod tests {
    use std::{env::current_dir, fs, process::Command};

    use libcommon::{
        curr_dir,
        ext::{FileDirCreateExt, PathJoinExt, PrettyStringExt},
        log::log_setup,
        prelude::{Result, info, timer},
    };

    #[timer]
    #[test]
    fn generate_dll() -> Result<()> {
        log_setup();
        let generate_dll = curr_dir!("target", "release", "plugin_adb.dll")?;
        let scan_dir_dll = current_dir()?
            .parent()
            .unwrap()
            .join_all(&["plugin_manager", "test_scan_dir", "plugin_adb.dll"])
            .create_parent()?;
        if generate_dll.exists() {
            std::fs::remove_file(&generate_dll)?;
        }
        let mut cmd = Command::new("cargo");
        cmd.args(&["build", "--release"]);
        let output = cmd.output()?;
        info!("execute: {} : {}", cmd.to_string_pretty(), output.status);

        assert!(generate_dll.exists());
        info!("target: {generate_dll:?}");

        if scan_dir_dll.exists() {
            fs::remove_file(&scan_dir_dll)?;
        }

        fs::copy(generate_dll, scan_dir_dll)?;

        Ok(())
    }
}
