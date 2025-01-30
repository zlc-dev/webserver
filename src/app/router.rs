use std::sync::Arc;
use tokio::io::AsyncBufRead;
use crate::http::{HttpRequest, HttpResponse, Method};
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

    pub async fn routing(&self, req: &mut HttpRequest<'_>) -> Result<HttpResponse, RouteError> {
        for route in &self.routes {
            if !route.methods.contains(req.header.method) {
                continue;
            }
            if let Some(captures) = route.pat.match_url(&req.header.url) {
                let res = route.next.handle(req, captures).await
                                    .map_err(|e| { RouteError::HandleError(e) })?;
                return Ok(res);
            }
        }
        Err(RouteError::NotFound)
    }

    pub fn register(&mut self, pattern: &str, handle: Arc<dyn EndPoint>, methods: MethodSet) -> Option<&mut Self> {
        self.routes.push(Route{pat: Pattern::from_str(pattern)?, methods, next: handle});
        Some(self)
    }
    
}


