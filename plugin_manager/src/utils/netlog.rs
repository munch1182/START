use axum::{
    body::{Body, to_bytes},
    extract::Request,
    http::StatusCode,
    response::Response,
};
use futures::future::BoxFuture;
use libcommon::{prelude::info, record};
use std::{str::from_utf8, time::Instant};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct LogLayer;

impl LogLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for LogLayer {
    type Service = LogServer<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LogServer { inner }
    }
}

#[derive(Clone)]
pub struct LogServer<S> {
    inner: S,
}

impl<S> Service<Request<Body>> for LogServer<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        Box::pin(async move {
            let version = req.version();
            let method = req.method().clone();
            let uri = req.uri().clone();
            let headers = req.headers().clone();
            let start = Instant::now();
            record!("--> {method} {uri} {version:?}");
            for (n, v) in headers.iter() {
                // sec-开头的是浏览器自动添加的
                if !n.to_string().starts_with("sec-") {
                    record!("{n}: {v:?}");
                }
            }
            // 克隆请求体以便读取
            let (parts, body) = req.into_parts();
            let bytes = to_bytes(body, 20 * 1024 * 1024).await.unwrap();
            if !bytes.is_empty()
                && let Ok(str) = from_utf8(&bytes)
            {
                record!("{str}");
            }
            record!("--> END {method}");
            let req = Request::from_parts(parts, Body::from(bytes));
            let res = inner.call(req).await?;
            let status = res.status();
            let elapsed = start.elapsed();
            let headers = res.headers().clone();
            record!("<-- {status} {uri} ({}ms)", elapsed.as_millis());
            for (n, v) in headers.iter() {
                record!("{n}: {v:?}");
            }
            let (parts, body) = res.into_parts();
            if status == StatusCode::OK {
                let bytes = to_bytes(body, 20 * 1024 * 1024).await.unwrap();
                if !bytes.is_empty()
                    && let Ok(str) = from_utf8(&bytes)
                {
                    record!("{str}");
                }
                record!("<-- END ({}-byte body)", bytes.len());
                Ok(Response::from_parts(parts, Body::from(bytes)))
            } else {
                record!("<-- END");
                Ok(Response::from_parts(parts, body))
            }
        })
    }
}
