use axum::{
    Json,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
pub struct Resp<T> {
    pub code: u16,
    pub msg: String,
    pub data: Option<T>,
}

impl<T: Serialize> IntoResponse for Resp<T> {
    fn into_response(self) -> Response {
        Json(json!(self)).into_response()
    }
}

impl<T> Resp<T> {
    pub fn sucess(data: T) -> Self {
        Self {
            code: 0,
            msg: String::default(),
            data: Some(data),
        }
    }

    pub fn is_success(&self) -> bool {
        self.code == 0
    }
}

impl Resp<()> {
    pub fn error<S: ToString>(code: u16, msg: S) -> Self {
        Self {
            code,
            msg: msg.to_string(),
            data: None,
        }
    }
}
