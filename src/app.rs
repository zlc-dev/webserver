use std::sync::Arc;

use tokio::{io, net::TcpListener, spawn, task::yield_now};

use crate::processor::{EndPoint, Processor};


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

    pub fn register(mut self, pattern: &str, handle: Arc<dyn EndPoint>) -> Option<Self> {
        self.processor.router.register(pattern, handle)?;
        Some(self)
    }


}
