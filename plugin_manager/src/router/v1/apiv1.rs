use crate::{
    config::FS_DIR_ROUTER,
    router::{
        APP_STATE, ApiImpl, AppState,
        v1::{admin::Admin, plugin::Plugin},
    },
    urlpath::UrlPath,
};
use axum::Router;
use std::{cell::RefCell, sync::Arc};
use tower_http::services::ServeDir;

pub struct ApiV1<'a> {
    prefix: &'a str,
    path: RefCell<UrlPath<'a>>,
}

impl<'a> ApiImpl<'a> for ApiV1<'a> {
    fn new(parent: &UrlPath<'a>) -> Self {
        let prefix = "/api/v1";
        Self {
            prefix,
            path: RefCell::new(parent.new_path_with(prefix)),
        }
    }

    fn router_str(&self) -> String {
        self.prefix.to_string()
    }

    fn router(&self) -> Router<Arc<AppState>> {
        let admin = Admin::new(&self.path.borrow());
        let plugin = Plugin::new(&self.path.borrow());
        let fs_dir = APP_STATE.wait().fs_dir();

        Router::new()
            .nest(&admin.router_str(), admin.router())
            .nest(&plugin.router_str(), plugin.router())
            .nest_service(FS_DIR_ROUTER, ServeDir::new(fs_dir))
    }
}
