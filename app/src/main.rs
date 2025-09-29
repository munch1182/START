#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // 禁用控制台窗口

use crate::config::Config;
use axum::Router;
#[cfg(not(debug_assertions))]
use include_dir::{Dir, include_dir};
use libcommon::prelude::*;
use plugin_manager::PluginManager;
use std::{
    env::current_dir,
    sync::{Arc, LazyLock},
};
use tokio::{net::TcpListener, sync::Notify};
use tower_http::{cors::CorsLayer, services::ServeDir};
use window::WindowManager;

mod config;
mod router;
mod utils;

#[derive(Clone)]
struct AppState {
    pm: Arc<PluginManager<Config>>,
}

#[cfg(not(debug_assertions))]
static RES_DIR: Dir = include_dir!("dist");

static WM: LazyLock<WindowManager> = LazyLock::new(WindowManager::default);

#[logsetup]
#[tokio::main]
async fn main() -> Result<()> {
    let mut config = Config::default();
    let server = TcpListener::bind(("0.0.0.0", port())).await?;
    let url = format!("http://{}", server.local_addr()?).replace("0.0.0.0", "127.0.0.1");

    let (net_base_dir, net_path_name) = {
        config.setup_dir(current_dir()?, &url);
        (config.net_base_dir.clone(), config.net_path_name.clone())
    };

    info!("listening on {url}, {url}/api, {url}/{net_path_name}");

    let app = Router::new()
        .nest("/api", router::api::routes())
        .nest_service(&format!("/{net_path_name}"), ServeDir::new(&net_base_dir))
        .layer(CorsLayer::permissive())
        .with_state(AppState::new(PluginManager::new(Arc::new(config))));

    let sd1 = Arc::new(Notify::new());
    let sd2 = sd1.clone();

    tokio::spawn(async move {
        info!("server start");
        if let Err(e) = axum::serve(server, app)
            .with_graceful_shutdown(async move { sd1.notified().await })
            .await
        {
            error!("server error: {e}");
            let _ = WM.exit();
        }
        info!("server exit");
    });

    {
        #[cfg(debug_assertions)]
        {
            let debug_server = "http://127.0.0.1:5173/";
            info!("server dir: {net_base_dir:?}; {debug_server}");
            WM.create_with_url("main", debug_server)?
        }
        #[cfg(not(debug_assertions))]
        {
            use libcommon::newerr;
            let html: &str = RES_DIR
                .get_file("index.html")
                .ok_or(newerr!("index.html not found in RES_DIR"))?
                .contents_utf8()
                .ok_or(newerr!("index.html contents_utf8 not found in RES_DIR"))?;
            let regex = regex::Regex::new(r#"(window\.SERVER_URL=")[^"]*(")"#)?;
            let html = regex.replace(html, &format!("window.SERVER_URL=\"{}\"", url));
            WM.create_with_html("main", html)?
        }
    }
    .on_close(move || sd2.notify_one())
    .run()
}

impl From<AppState> for Arc<PluginManager<Config>> {
    fn from(val: AppState) -> Self {
        val.pm
    }
}

impl AppState {
    pub fn new(pm: PluginManager<Config>) -> Self {
        Self { pm: Arc::new(pm) }
    }
}

#[cfg(debug_assertions)]
fn port() -> u16 {
    12321
}

#[cfg(not(debug_assertions))]
fn port() -> u16 {
    0
}
