use crate::pm::PluginManager;
use axum::Router;
use libcommon::prelude::*;
use std::{
    env::current_dir,
    sync::{Arc, LazyLock},
};
use tokio::net::TcpListener;
use window::WindowManager;

mod config;
mod pm;
mod router;

pub use config::CONFIG;

#[derive(Clone, Default)]
struct AppState {
    pm: Arc<PluginManager>,
}

#[cfg(debug_assertions)]
fn port() -> u16 {
    12345
}

#[cfg(not(debug_assertions))]
fn port() -> u16 {
    1234
}

static WM: LazyLock<WindowManager> = LazyLock::new(WindowManager::default);

#[logsetup]
#[tokio::main]
async fn main() -> Result<()> {
    let server = TcpListener::bind(("127.0.0.1", port())).await?;
    let url = format!("http://{}", server.local_addr()?);
    info!("listening on {url}");
    let app = Router::new()
        .nest("/api", router::api::routes())
        .with_state(AppState::default());

    let curr_dir = current_dir()?;
    {
        CONFIG.write().newerr()?.setup_dir(curr_dir);
    }

    tokio::spawn(async move {
        if let Err(e) = axum::serve(server, app).await {
            error!("server error: {e}");
            let _ = WM.exit();
        }
    });
    WM.create_with_url("main", url)?.run()
}

impl From<AppState> for Arc<PluginManager> {
    fn from(val: AppState) -> Self {
        val.pm
    }
}
