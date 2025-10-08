use crate::{
    search::{self, SearchResultItem},
    utils::opt::OptParam,
};
use axum::{Router, routing::get};
use plugin_d::Resp;
use serde::Deserialize;

pub fn routes<T>() -> Router<T>
where
    T: Clone + Send + Sync + 'static,
{
    Router::<T>::new().route("/s", get(search))
}

async fn search(OptParam(q): OptParam<Search>) -> Resp<Vec<SearchResultItem>> {
    search::search(q.map(|q| q.q)).into()
}

#[derive(Deserialize)]
struct Search {
    pub q: String,
}
