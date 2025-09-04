pub mod router;
pub mod urlpath;
mod utils;

use libcommon::prelude::{Result, info};
use serde::Serializer;
use tokio::net::TcpListener;

use crate::{router::AppRouter, utils::netlog::LogLayer};

pub struct App {
    host: String,
    _lis: TcpListener,
}

impl App {
    pub async fn new() -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let host = listener.local_addr()?;
        Ok(Self {
            host: format!("http://{host}"),
            _lis: listener,
        })
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub async fn run(self) -> Result<()> {
        let server = self.host;
        info!("Starting server at {server}");
        let app_router = AppRouter::new(&server);
        let router = app_router.router().layer(LogLayer::new());
        axum::serve(self._lis, router).await?;
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

#[cfg(test)]
mod tests {
    use libcommon::{
        log::log_setup,
        prelude::{info, timer},
    };

    use super::*;

    #[timer]
    #[tokio::test]
    async fn test_port() {
        log_setup();
        let a = App::new().await.unwrap();
        info!("{a}");
        assert!(!a.host.split(':').last().unwrap().is_empty());
    }
}
