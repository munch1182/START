use crate::{
    router::{ApiImpl, info_router},
    urlpath::UrlPath,
};
use axum::{Router, routing::get};
use std::{cell::RefCell, sync::Arc};

pub(crate) struct Plugin<'a> {
    prefix: &'a str,
    path: RefCell<UrlPath<'a>>,
}

impl<'a> ApiImpl<'a> for Plugin<'a> {
    fn new(parent: &UrlPath<'a>) -> Self {
        let prefix = "/plugin";
        Self {
            prefix,
            path: RefCell::new(parent.new_path_with(prefix)),
        }
    }

    fn router_str(&self) -> String {
        self.prefix.to_string()
    }

    fn router(&self) -> Router<Arc<super::AppState>> {
        let list = self.path.borrow().new_path_with("/list");
        info_router(&list);
        Router::new().route(list.curr_part().unwrap_or_default(), get(scan))
    }
}

async fn scan() -> String {
    "scan".to_string()
}
