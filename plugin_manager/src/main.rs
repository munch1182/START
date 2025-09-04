use libcommon::prelude::*;
use plugin_manager::App;

#[logsetup("testlogdir")]
#[tokio::main]
pub async fn main() -> Result<()> {
    App::new().await?.run().await?;
    Ok(())
}
