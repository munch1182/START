use libcommon::prelude::*;
use plugin_manager::App;
use plugin_manager::router::router;
use plugin_manager::urlpath::UrlPath;

#[logsetup("testlogdir")]
#[tokio::main]
pub async fn main() -> Result<()> {
    let app = App::new().await?;
    info!("Starting serve: {app}");

    let url = app.to_string();
    let mut path = UrlPath::new(&url);

    app.run(router(&mut path)).await?;
    Ok(())
}
