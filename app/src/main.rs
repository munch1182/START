mod cmd;
mod context;
mod server;
use crate::{
    cmd::{callplugin, listplugins, scan},
    server::Server,
};
use libcommon::{New, prelude::*};
use pluginmanager::PluginManager;
use window::{WindowCreateExt, WindowManager, generate};

#[tokio::main]
#[logsetup(level = trace)]
async fn main() -> Result<()> {
    let server = Server::new(3030);
    let url = server.window_url();
    let server2 = server.clone();

    tokio::spawn(async move {
        match server2.run().await {
            Ok(_) => debug!("server stopped"),
            Err(e) => error!("server start failed: {e}"),
        }
    });

    let file_dir = std::env::current_dir()?
        .join("dist")
        .to_string_lossy()
        .to_string();
    let pm = PluginManager::default();
    let state = AppState::new(pm, server, file_dir);
    let wm = WindowManager::with_state(state.into());
    wm.create_window("main", url)?;
    wm.register_handler(generate!(listplugins, scan, callplugin));
    info!("launch window");
    wm.run()
}

#[derive(New)]
pub struct AppState {
    pub pm: PluginManager,
    pub server: Server,
    pub plugin_dir: String,
}
