use axum::Router;
use std::{cell::RefCell, sync::Arc};

use crate::{
    router::{ApiImpl, admin::Admin},
    urlpath::UrlPath,
};

pub(crate) struct ApiV1<'a> {
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

    fn router(&self) -> Router<Arc<super::AppState>> {
        let admin = Admin::new(&self.path.borrow());
        Router::new().nest(&admin.router_str(), admin.router())
    }
}
