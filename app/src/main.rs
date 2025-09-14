use std::sync::OnceLock;

use libcommon::{curr_dir, prelude::*};
use plugin_manager::{App as AppNet, router::AppConfig};
use window::{App as AppWindow, WindowConfig, WindowCreateExt, WindowExecuteExt};

static APP: OnceLock<AppWindow> = OnceLock::new();

#[logsetup("test_log_dir")]
#[tokio::main]
async fn main() -> Result<()> {
    let app = AppWindow::default();
    let _ = APP.set(app);
    let app_net = AppNet::new().await?;
    let url = app_net.host();
    tokio::spawn(async move {
        let scan_dir = curr_dir!("test_scan_dir").unwrap();
        let fs_dir = curr_dir!("test_fs_dir").unwrap();
        app_net.run(AppConfig::new(scan_dir, fs_dir)).await.unwrap();
    });
    let w = 400;
    let h = 120;
    APP.wait()
        .create(
            WindowConfig::new("main")
                .with_url(url)
                .with_size(w, h)
                .with_position((2560 - w) / 2, (1440 - h) / 2),
        )?
        .run()
}
