use async_trait::async_trait;

use crate::http::{HttpRequest, HttpResponse};
use std::{future::Future, pin::Pin};

pub type HttpResult = Result<HttpResponse, super::HandleError>;

#[async_trait]
pub trait EndPoint: Send + Sync + 'static {
    async fn handle<'a, 'b>(&self, req: &'a mut HttpRequest<'b>, captures: Vec<&'b str> ) -> HttpResult;
}


#[async_trait]
impl<F> EndPoint for F
where
    F: Send
        + Sync
        + 'static
        + for<'a, 'b> Fn(
            &'a mut HttpRequest<'b>,
            Vec<&'b str>
        ) -> Pin<Box<dyn Future<Output = HttpResult> + 'a + Send>>,
{
    async fn handle<'a, 'b>(&self, req: &'a mut HttpRequest<'b>, captures: Vec<&'b str>) -> HttpResult {
        (self)(req, captures).await
    }
}

#[macro_export]
macro_rules! ep_wrap{
    ($f:expr) => {
        {
            fn ep_wrap_f<'a, 'b>(req: &'a mut HttpRequest<'b>, captures: Vec<&'b str>) -> Pin<Box<dyn Future<Output = HttpResult> + 'a + Send>>{
                Box::pin($f(req, captures))
            }
            Arc::new(ep_wrap_f)
        }
    };
}
