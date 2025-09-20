use axum::{
    Json,
    response::{IntoResponse, Response},
};
use libcommon::prelude::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Resp<T> {
    pub code: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

unsafe impl<T> Send for Resp<T> {}

impl<T> Resp<T> {
    #[inline]
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            msg: None,
            data: Some(data),
        }
    }

    #[inline]
    pub fn error(code: u8, msg: String) -> Self {
        Self {
            code,
            msg: Some(msg),
            data: None,
        }
    }
}

impl<T: Serialize> IntoResponse for Resp<T> {
    fn into_response(self) -> Response {
        (Json(self)).into_response()
    }
}

impl<T> From<T> for Resp<T> {
    fn from(value: T) -> Self {
        Self::success(value)
    }
}
impl<T> From<Result<T>> for Resp<T> {
    fn from(value: Result<T>) -> Self {
        match value {
            Ok(v) => Self::success(v),
            Err(e) => Self::error(1, format!("{e:?}")),
        }
    }
}
