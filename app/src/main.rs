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
use window::TaoWindow;
use window::TaoWindowExt;
use window::WindowFindExt;
use window::register_key;
use window::{TaoWindowBuilder, WindowConfig, WindowManager};

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

    listen_key();

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

    let cfg = new_window("main")
        .with_page(page)
        .with_webview(InitJs::default_with(url).init());
    WM.create(cfg)?.on_close(move || sd2.notify_one()).run()
}

fn new_window(name: impl ToString) -> WindowConfig {
    let with_w = |w: TaoWindowBuilder| {
        w.with_decorations(false)
            .with_always_on_top(true)
            .with_transparent(true)
            .with_skip_taskbar(true)
    };
    WindowConfig::new(name)
        .with_size((816, 56 * 7))
        .with_window(with_w)
}

fn listen_key() {
    register_key("ControlLeft+ControlLeft", || {
        if let Some(w) = WM.curr() {
            let _ = WM.find(w.label.as_str(), |w: &TaoWindow| {
                w.set_visible(!w.is_visible())
            });
        }
    });
    register_key("Escape", || {
        WM.curr().and_then(|x| x.hide());
    });
    // window::register_any_key(|k| {
    //     info!("key: {k:?}");
    // });
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

fn port() -> u16 {
    #[cfg(debug_assertions)]
    {
        12321
    }
    #[cfg(not(debug_assertions))]
    {
        0
    }
}
