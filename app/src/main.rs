#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // 禁用控制台窗口

use crate::config::Config;
use crate::config::InitJs;
use crate::search::SearchItem;
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
use window::WindowConfig;
use window::{TaoWindowBuilder, WindowManager};

mod config;
mod router;
mod search;
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

    let app_state = AppState::new(PluginManager::new(Arc::new(config)));

    let app = Router::new()
        .nest("/api", router::api::routes())
        .nest_service(&format!("/{net_path_name}"), ServeDir::new(&net_base_dir))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

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

    let page = {
        #[cfg(debug_assertions)]
        {
            let debug_server = "http://127.0.0.1:5173/";
            info!("server dir: {net_base_dir:?}; {debug_server}");
            debug_server
        }
        #[cfg(not(debug_assertions))]
        {
            RES_DIR.get_file("index.html")?.contents_utf8()?
        }
    };

    let with_w = |w: TaoWindowBuilder| w.with_decorations(false).with_always_on_top(true);
    let cfg = WindowConfig::new("main")
        .with_page(page)
        .with_size((816, 56 * 7))
        .with_webview(InitJs::default_with(url).init())
        .with_window(with_w);

    WM.create(cfg)?.on_close(move || sd2.notify_one()).run()
}

impl AppState {
    pub fn new(pm: PluginManager<Config>) -> Self {
        let pm = pm.set_on_update(|p| {
            search::on_update(
                p.into_iter()
                    .map(|p| SearchItem::new(&p.id, &p.info.name, p.info.keyword.clone()))
                    .collect::<Vec<_>>(),
            )
        });
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
