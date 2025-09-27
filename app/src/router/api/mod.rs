use crate::AppState;
use axum::Router;

pub(crate) mod plugin;
pub(crate) mod v1 {
    use crate::{AppState, router::api::plugin};
    use axum::Router;

    /// `path`: /api/v1/plugin/*
    pub fn routes() -> Router<AppState> {
        Router::new().nest("/v1/plugin", plugin::routes())
    }
}

/// `path`: /api/*
pub fn routes() -> Router<AppState> {
    v1::routes()
}
