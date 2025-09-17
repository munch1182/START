use libcommon::{curr_dir, ext::FileDirCreateExt, prelude::*};
use plugin_manager::{App, router::AppConfig};

#[logsetup("test_log_dir")]
#[tokio::main]
pub async fn main() -> Result<()> {
    let scan_dir = curr_dir!("test_scan_dir")?.create_dir()?;
    let fs_dir = curr_dir!("test_fs_dir")?.create_dir()?;
    let config = AppConfig::new(scan_dir, fs_dir);
    App::new(config).await?.run().await?;
    Ok(())
}
