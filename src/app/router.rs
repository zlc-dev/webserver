use std::sync::Arc;
use tokio::io::AsyncBufRead;
use crate::http::{HttpRequest, HttpRequestLine, HttpResponse, Method};
use super::{EndPoint, Pattern, RouteError};

#[derive(Clone, Copy)]
pub struct MethodSet {
    bits: u8,
}

impl MethodSet {

    pub fn new() -> Self{
        MethodSet{
            bits: 0_u8
        }
    }

    pub fn insert(&mut self, method: Method) {
        self.bits |= 1_u8 << (method as u8);
    }

    pub fn remove(&mut self, method: Method) {
        self.bits &= !(1_u8 << (method as u8));
    }

    pub fn contains(&self, method: Method) -> bool {
        self.bits & (1_u8 << (method as u8)) != 0
    }
}

pub struct Route {
    pub pat: Pattern,
    pub methods: MethodSet,
    pub next: Arc<dyn EndPoint>,
}

pub struct Router {
    pub routes: Vec<Route>,
    // pub sub: HashMap<&'static str, Router>,
}

impl Router {

    pub fn new() -> Self {
        Router {
            routes: Vec::new(),
        }
    }

    pub async fn handle(&self, mut buf_reader: impl AsyncBufRead + Unpin + Send) -> Result<HttpResponse, RouteError> {

        let request_line = HttpRequestLine::from_async_stream(&mut buf_reader).await.map_err(|e| { RouteError::ReqError(e) })?;

        for route in &self.routes {
            if !route.methods.contains(request_line.method) {
                continue;
            }
            if let Some(args) = route.pat.match_url(&request_line.url) {
                let mut req = HttpRequest::create(&request_line, args, &mut buf_reader);
                let res = route.next.handle(&mut req).await
                                    .map_err(|e| { RouteError::HandleError(e) })?;
                return Ok(res);
            }
        }
        
        Ok(HttpResponse::create_404_not_found())
    }

    pub fn register(&mut self, pattern: &str, handle: Arc<dyn EndPoint>, methods: MethodSet) -> Option<&mut Self> {
        self.routes.push(Route{pat: Pattern::from_str(pattern)?, methods, next: handle});
        Some(self)
    }
    
}


