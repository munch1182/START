use axum::{
    Json,
    response::{IntoResponse, Response},
};
use libcommon::prelude::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
pub struct Resp<T> {
    pub code: u16,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> IntoResponse for Resp<T> {
    fn into_response(self) -> Response {
        Json(json!(self)).into_response()
    }
}

impl<T> From<Result<T>> for Resp<T> {
    fn from(value: Result<T>) -> Self {
        match value {
            Ok(data) => Self::sucess(data),
            Err(err) => Self {
                code: 1,
                msg: format!("{err:?}"),
                data: None,
            },
        }
    }
}

impl<T> Resp<T> {
    #[inline]
    pub fn sucess(data: T) -> Self {
        Self {
            code: 0,
            msg: String::default(),
            data: Some(data),
        }
    }

    #[inline]
    pub fn is_success(&self) -> bool {
        self.code == 0
    }

    #[inline]
    pub fn error_with(code: u16, msg: &str) -> Self {
        Self {
            code,
            msg: msg.to_string(),
            data: None,
        }
    }
}

impl Resp<()> {
    #[inline]
    pub fn error<S: ToString>(code: u16, msg: S) -> Self {
        Self {
            code,
            msg: msg.to_string(),
            data: None,
        }
    }
}
