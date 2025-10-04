use crate::{AppState, config::Config};
use axum::Router;
use plugin_manager::PluginManager;
use std::sync::Arc;

pub(crate) mod plugin;
pub(crate) mod search;
pub(crate) mod v1 {
    use crate::{
        AppState,
        router::api::{plugin, search},
    };
    use axum::Router;

    /// `path`: /api/*
    pub fn routes() -> Router<AppState> {
        Router::new()
            .nest("/v1/plugin", plugin::routes())
            .nest("/v1/search", search::routes())
    }
}

/// `path`: /api/*
pub fn routes() -> Router<AppState> {
    v1::routes()
}

impl From<AppState> for Arc<PluginManager<Config>> {
    fn from(val: AppState) -> Self {
        val.pm
    }
}
