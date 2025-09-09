use libcommon::{curr_dir, prelude::*};
use plugin_manager::{App, router::AppConfig};

#[logsetup("testlogdir")]
#[tokio::main]
pub async fn main() -> Result<()> {
    let scan_dir = curr_dir!("test_scan_dir")?;
    let config = AppConfig::new(scan_dir);
    App::new().await?.run(config).await?;
    Ok(())
}
