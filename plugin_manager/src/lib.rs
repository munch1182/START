pub mod config;
mod pm;
mod respres;
pub mod router;
pub mod urlpath;
mod utils;

use crate::{
    router::{APP_STATE, AppConfig, AppRouter, AppState},
    utils::netlog::LogLayer,
};
use libcommon::{
    newerr,
    prelude::{Result, info},
};
use serde::Serializer;
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct App {
    host: String,
    lis: TcpListener,
}

impl App {
    pub async fn new(config: AppConfig) -> Result<Self> {
        let listener = {
            #[cfg(debug_assertions)]
            {
                TcpListener::bind("127.0.0.1:1234").await?
            }
            #[cfg(not(debug_assertions))]
            {
                TcpListener::bind("127.0.0.1:0").await?
            }
        };
        APP_STATE
            .set(Arc::new(AppState::new(config)))
            .map_err(|_| newerr!("app state already set"))?;
        let host = listener.local_addr()?;
        Ok(Self {
            host: format!("http://{host}"),
            lis: listener,
        })
    }

    pub async fn new_with_scan(config: AppConfig) -> Result<Self> {
        let app = Self::new(config).await?;
        app.scan();
        Ok(app)
    }

    pub fn host(&self) -> String {
        self.host.to_string()
    }

    pub fn scan(&self) {
        if let Some(app) = APP_STATE.get() {
            let _ = app.pm().scan();
        }
    }

    pub async fn run(self) -> Result<()> {
        let server = self.host;
        info!("Starting server at {server}");
        let app_router = AppRouter::new(&server);
        let router = app_router.router().layer(LogLayer::new());
        axum::serve(self.lis, router).await?;
        Ok(())
    }
}

impl std::fmt::Display for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.is_human_readable() {
            write!(f, "{}", self.host)
        } else {
            write!(f, "App({})", self.host)
        }
    }
}

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.is_human_readable() {
            write!(f, "{}", self.host)
        } else {
            write!(f, "App({})", self.host)
        }
    }
}
