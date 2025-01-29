use async_trait::async_trait;

use crate::http::{HttpRequest, HttpResponse};
use std::{future::Future, pin::Pin};

pub type HttpResult = Result<HttpResponse, super::Error>;

#[async_trait]
pub trait EndPoint: Send + Sync + 'static {
    async fn handle(&self, req: HttpRequest<'_>) -> HttpResult;
}


#[async_trait]
impl<F> EndPoint for F
where
    F: Send
        + Sync
        + 'static
        + for<'a> Fn(
            HttpRequest<'a>,
        ) -> Pin<Box<dyn Future<Output = HttpResult> + 'a + Send>>,
{
    async fn handle(&self, req: HttpRequest<'_>) -> HttpResult {
        (self)(req).await
    }
}



