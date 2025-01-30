use async_trait::async_trait;

use crate::http::{HttpRequest, HttpResponse};
use std::{future::Future, pin::Pin};

pub type HttpResult = Result<HttpResponse, super::HandleError>;

#[async_trait]
pub trait EndPoint: Send + Sync + 'static {
    async fn handle<'a, 'b>(&self, req: &'a mut HttpRequest<'b> ) -> HttpResult;
}


#[async_trait]
impl<F> EndPoint for F
where
    F: Send
        + Sync
        + 'static
        + for<'a, 'b> Fn(
            &'a mut HttpRequest<'b>,
        ) -> Pin<Box<dyn Future<Output = HttpResult> + 'a + Send>>,
{
    async fn handle<'a, 'b>(&self, req: &'a mut HttpRequest<'b>) -> HttpResult {
        (self)(req).await
    }
}

#[macro_export]
macro_rules! ep_wrap{
    ($f:expr) => {
        {
            fn ep_wrap_f<'a, 'b>(req: &'a mut HttpRequest<'b>) -> Pin<Box<dyn Future<Output = HttpResult> + 'a + Send>>{
                Box::pin($f(req))
            }
            Arc::new(ep_wrap_f)
        }
    };
}
