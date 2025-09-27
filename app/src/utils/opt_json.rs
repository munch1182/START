use std::ops::{Deref, DerefMut};

use axum::{
    Json,
    extract::{FromRequest, Request},
    response::Response,
};
use serde::de::DeserializeOwned;

/// 使用Option<Json<T>>会在传入header为json时失败
#[derive(Debug, Clone, Copy, Default)]
pub struct OptJson<T>(pub Option<T>);

impl<T> Deref for OptJson<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for OptJson<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<Option<T>> for OptJson<T> {
    fn from(option: Option<T>) -> Self {
        OptJson(option)
    }
}

impl<T, S> FromRequest<S> for OptJson<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // 尝试解析 JSON
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(OptJson(Some(value))),
            Err(_) => Ok(OptJson(None)), // 解析失败也返回 None
        }
    }
}
