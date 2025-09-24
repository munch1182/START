use crate::pm::PluginManager;
use axum::Router;
use libcommon::prelude::*;
use std::{
    env::current_dir,
    sync::{Arc, LazyLock},
};
use tokio::net::TcpListener;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use window::WindowManager;

mod config;
mod pm;
mod router;
mod utils;

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
    let server = TcpListener::bind(("0.0.0.0", port())).await?;
    let url = format!("http://{}", server.local_addr()?);
    let curr_dir = current_dir()?;
    let (server_dir, serve_dir_name) = {
        let mut config = CONFIG.write().newerr()?;
        config.setup_dir(curr_dir);
        (config.dir_fs.clone(), config.name_file_net.clone())
    };

    info!("listening on {url}");
    let app = Router::new()
        .nest("/api", router::api::routes())
        .nest_service(&format!("/{serve_dir_name}"), ServeDir::new(&server_dir))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(AppState::default());

    tokio::spawn(async move {
        if let Err(e) = axum::serve(server, app).await {
            error!("server error: {e}");
            let _ = WM.exit();
        }
    });
    let index = {
        #[cfg(debug_assertions)]
        {
            "http://127.0.0.1:5173/".to_string()
        }
        #[cfg(not(debug_assertions))]
        {
            format!("{url}/{serve_dir_name}/launcher/index.html")
        }
    };
    info!("server dir: {server_dir:?} => {index}");
    WM.create_with_url("main", index)?.run()
}

impl From<AppState> for Arc<PluginManager> {
    fn from(val: AppState) -> Self {
        val.pm
    }
}
