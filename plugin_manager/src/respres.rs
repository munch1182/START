use axum::response::{IntoResponse, Response};
use libcommon::prelude::{Result, warn};
use plugin_d::resp::Resp;
use serde::Serialize;

///
/// 允许错误的响应包装
///
/// 将Ok(T)和Err(E)转换为Resp<T>和Resp<E>
///
pub(crate) struct RespResult<T: Serialize> {
    pub(crate) inner: Result<T>,
}

impl<T: Serialize> From<Result<T>> for RespResult<T> {
    fn from(value: Result<T>) -> Self {
        Self { inner: value }
    }
}

impl<T: Serialize> From<T> for RespResult<T> {
    fn from(value: T) -> Self {
        Self { inner: Ok(value) }
    }
}

impl<T> IntoResponse for RespResult<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        match self.inner {
            Ok(r) => Resp::sucess(r).into_response(),
            Err(e) => {
                warn!("error to response: {e}");
                Resp::<T>::error_with(1, "error").into_response()
            }
        }
    }
}
