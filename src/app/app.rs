use std::sync::Arc;

use tokio::{io, net::TcpListener, spawn, task::yield_now};

use crate::{http::Method, app::{EndPoint, MethodSet, Processor}};


pub struct Application {
    pub listeners: Vec<TcpListener>,
    pub processor: Processor,
}

impl Application {

    pub fn new() -> Self {
        Application {
            listeners: Vec::new(),
            processor: Processor::new(),
        }
    }

    pub async fn run(self) {
        let processor = Arc::new(self.processor);
        for listener in self.listeners {
            let loc_processor = processor.clone();
            spawn(async move {
                loop {
                    if let Ok((stream, _)) = listener.accept().await {
                        let loc_processor = loc_processor.clone();
                        spawn(async move {
                            if let Err(e) = loc_processor.handle(stream).await {
                                println!("Handle Error: {:?}", e);
                            }
                        });
                    }
                }
            });
        }
        loop {
            yield_now().await;
        }
    }

    pub async fn listen_tcp(mut self, addr:&str) -> io::Result<Self>{
        self.listeners.push(TcpListener::bind(addr).await?);
        Ok(self)
    }

    pub fn register<'a>(mut self, pat: &'a str, handle: Arc<dyn EndPoint>, methods: MethodSet) -> Self{
        self.processor.router.register(pat, handle, methods);
        self
    }

    pub fn registrar(self) -> RouteRegistrar {
        RouteRegistrar::new(self)
    }


}


pub struct RouteRegistrar {
    app: Application,
    methods: MethodSet,
    prefix: String,
}

impl RouteRegistrar {

    pub fn new(app: Application) -> Self {
        RouteRegistrar {
            app,
            methods: MethodSet::new(),
            prefix: String::new(),
        }
    }

    pub fn app(self) -> Application {
        self.app
    }

    pub fn register<'b>(mut self, pat: &'b str, handle: Arc<dyn EndPoint>) -> Self{
        self.app = self.app.register(format!("{}{}", self.prefix, pat).as_str(), handle, self.methods);
        self
    }

    pub fn at<'b>(mut self, pre: &'b str) -> Self {
        self.prefix.push_str(pre);
        self
    }

    pub fn get(mut self) -> Self {
        self.methods.insert(Method::GET);
        self
    }

    pub fn post(mut self) -> Self {
        self.methods.insert(Method::POST);
        self
    }

    pub fn patch(mut self) -> Self {
        self.methods.insert(Method::PATCH);
        self
    }

    pub fn put(mut self) -> Self {
        self.methods.insert(Method::PUT);
        self
    }

    pub fn delete(mut self) -> Self {
        self.methods.insert(Method::DELETE);
        self
    }

    pub fn head(mut self) -> Self {
        self.methods.insert(Method::HEAD);
        self
    }

    pub fn options(mut self) -> Self {
        self.methods.insert(Method::OPTIONS);
        self
    }

    pub fn trace(mut self) -> Self {
        self.methods.insert(Method::TRACE);
        self
    }

}
