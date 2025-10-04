use axum::{
    Json,
    extract::{FromRequest, Query, Request},
    response::Response,
};
use serde::de::DeserializeOwned;
use std::ops::{Deref, DerefMut};

/// 使用Option<Json<T>>会在传入header为json时失败
#[derive(Debug, Clone, Copy, Default)]
pub struct OptParam<T>(pub Option<T>);

impl<T> Deref for OptParam<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for OptParam<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<Option<T>> for OptParam<T> {
    fn from(option: Option<T>) -> Self {
        OptParam(option)
    }
}

impl<T, S> FromRequest<S> for OptParam<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let uri = req.uri().clone();

        if let Ok(Json(value)) = Json::<T>::from_request(req, state).await {
            return Ok(OptParam(Some(value)));
        }

        if let Ok(req) = Request::builder().uri(uri).body(axum::body::Body::empty())
            && let Ok(query) = Query::<T>::from_request(req, state).await
        {
            return Ok(OptParam(Some(query.0)));
        }

        Ok(OptParam(None))
    }
}
