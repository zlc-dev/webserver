use std::sync::Arc;

use tokio::net::tcp::OwnedReadHalf;

use crate::http::{ReqError, HttpRequest, HttpRequestLine, HttpResponse};

use super::{EndPoint, Pattern, Error};

pub struct Route {
    pub pat: Pattern,
    pub next: Arc<dyn EndPoint>,
}

pub struct Router {
    pub routes: Vec<Route>,
    // pub sub: HashMap<&'static str, Router>,
}


#[derive(Debug)]
pub enum RouteError {
    Error(Error),
    ReqError(ReqError),
    NoMatchError,
}

impl Router {

    pub fn new() -> Self {
        Router {
            routes: Vec::new(),
        }
    }

    pub async fn handle(&self, mut rx: OwnedReadHalf) -> Result<HttpResponse, RouteError> {

        let request_line = HttpRequestLine::from_async_stream(&mut rx).await.map_err(|e| { RouteError::ReqError(e) })?;

        for route in &self.routes {
            if let Some(args) = route.pat.match_url(&request_line.url) {
                return Ok(route.next.handle(HttpRequest::create(&request_line, args, &mut rx)).await
                                    .map_err(|e| { RouteError::Error(e) })?);
            }
        }
        return Ok(HttpResponse::create_404_not_found());
    }

    pub fn register(&mut self, pattern: &str, handle: Arc<dyn EndPoint>) -> Option<&mut Self> {
        self.routes.push(Route{pat: Pattern::from_str(pattern)?, next: handle});
        Some(self)
    }
    
}

#[macro_export]
macro_rules! boxed_f{
    ($f:expr) => {
        {
            fn boxed_f<'a>(req: HttpRequest<'a>) -> Pin<Box<dyn Future<Output = HttpResult> + 'a + Send>>{
                Box::pin($f(req))
            }
            boxed_f
        }
    };
}
