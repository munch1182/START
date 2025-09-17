use libcommon::{curr_dir, prelude::*};
use plugin_manager::{App as AppNet, router::AppConfig};
use std::sync::LazyLock;
use window::{App as AppWindow, WindowConfig, WindowCreateExt, WindowExecuteExt};

static APP: LazyLock<AppWindow> = LazyLock::new(AppWindow::default);

#[logsetup("test_log_dir")]
#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::new(curr_dir!("test_scan_dir")?, curr_dir!("test_fs_dir")?);
    let app_net = AppNet::new_with_scan(config).await?;
    let url = app_net.host();
    tokio::spawn(async {
        if app_net.run().await.is_err() {
            APP.exit();
        }
    });
    let w = 500;
    let h = 200;
    APP.create(
        WindowConfig::new("main")
            .with_url(url)
            .with_size((w, h))
            .with_position(((2560 - w) / 2, (1440 - h) / 2)),
    )?
    .run()
}
