use libcommon::{curr_dir, prelude::*};
use plugin_manager::{App as AppNet, router::AppConfig};
use std::sync::LazyLock;
use window::{App as AppWindow, WindowConfig, WindowCreateExt, WindowExecuteExt};

static APP: LazyLock<AppWindow> = LazyLock::new(AppWindow::default);

#[logsetup("test_log_dir")]
#[tokio::main]
async fn main() -> Result<()> {
    let app_net = AppNet::new().await?;
    let url = app_net.host();
    let scan_dir = curr_dir!("test_scan_dir")?;
    let fs_dir = curr_dir!("test_fs_dir")?;
    tokio::spawn(async {
        if let Err(_) = app_net.run(AppConfig::new(scan_dir, fs_dir)).await {
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
