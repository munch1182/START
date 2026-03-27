use std::process::Command;

use libcommon::prelude::*;
use window::{WindowCreateExt, WindowManager, bridge, generate};

#[tokio::main]
#[logsetup]
async fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    start_dev_server();

    let wm = WindowManager::default();
    wm.create_window("main", "http://localhost:3000/")?;
    wm.register_handler(generate!(select));
    wm.run()
}

#[bridge]
async fn select(id: String, name: String) -> Result<bool> {
    debug!("select: {id:?}, {name:?}");
    Ok(false)
}

fn start_dev_server() {
    #[cfg(target_os = "windows")]
    tokio::spawn(async {
        let result = Command::new("cmd")
            .current_dir("./app")
            .args(["/c pnpm run dev"])
            .output();
        debug!("dev server result: {result:?}");
    });
}
