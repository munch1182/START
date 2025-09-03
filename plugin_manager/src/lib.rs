pub mod router;
pub mod urlpath;

use axum::Router;
use libcommon::prelude::Result;
use serde::Serializer;
use tokio::net::TcpListener;

pub struct App {
    addr: String,
    _lis: TcpListener,
}

impl std::fmt::Display for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.is_human_readable() {
            write!(f, "http://{}", self.addr)
        } else {
            write!(f, "App(http://{})", self.addr)
        }
    }
}

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.is_human_readable() {
            write!(f, "http://{}", self.addr)
        } else {
            write!(f, "App(http://{})", self.addr)
        }
    }
}

impl App {
    pub async fn new() -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        Ok(Self {
            addr: addr.to_string(),
            _lis: listener,
        })
    }

    pub fn addr(&self) -> &str {
        &self.addr
    }

    pub async fn run(self, app: Router) -> Result<()> {
        axum::serve(self._lis, app).await?;
        Ok(())
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
    async fn test_addr() {
        log_setup();
        let a = App::new().await.unwrap();
        info!("{a}");
        assert!(!a.addr.split(':').last().unwrap().is_empty());
    }
}
