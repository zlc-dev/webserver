use std::{sync::{atomic::AtomicU64, Arc}, time::Duration};
use tokio::{io::{self, BufReader}, net:: TcpListener, spawn, sync::Mutex, task::yield_now, time::Instant};
use crate::{http::Method, app::{EndPoint, MethodSet, Processor}};

use super::processor;

pub struct Application {
    pub listeners: Vec<TcpListener>,
    pub processor: Processor,
    pub conns_count: AtomicU64,
}

impl Application {

    pub fn new() -> Self {
        Application {
            listeners: Vec::new(),
            processor: Processor::new(),
            conns_count: AtomicU64::new(0),
        }
    }

    pub async fn run(self) {
        let processor = Arc::new(self.processor);
        let conns_count = Arc::new(self.conns_count);
        for listener in self.listeners {
            let loc_processor = processor.clone();
            let conns_count = conns_count.clone();
            spawn(async move {
                loop {
                    if let Ok((stream, _)) = listener.accept().await {
                        let loc_processor = loc_processor.clone();
                        let conns_count = conns_count.clone();
                        spawn(async move {
                            println!("Connection Create");
                            conns_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            let (rx, mut tx) = stream.into_split();
                            let mut buf_rx = BufReader::new(rx);
                            let start_instant = Instant::now();
                            while conns_count.load(std::sync::atomic::Ordering::Relaxed) < 20
                                    // very bad
                                    && start_instant + Duration::from_secs(5) >= Instant::now() {
                                match loc_processor.handle(&mut buf_rx, &mut tx).await {
                                    Ok(resp) => {
                                        match resp.connect_state {
                                            super::ConnectionState::Opening => {},
                                            super::ConnectionState::Closed => break,
                                        }
                                    },
                                    Err(e) => println!("Handle Error: {:?}", e),
                                }
                            }
                            println!("Connection End");
                            conns_count.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                        });
                    }
                }
            });
        }
        // connection_pool.run(processor, connection_sender).await;
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
