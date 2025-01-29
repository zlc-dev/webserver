use tokio::{io::{self}, net::TcpStream};

use super::{RouteError, Router};

pub struct Processor {
    pub router: Router
}

#[derive(Debug)]
pub enum ProcError {
    RouteError(RouteError),
    IoError(io::Error),
}

impl Processor{

    pub fn new() -> Self {
        Processor {
            router: Router::new(),
        }
    }

    pub async fn handle(&self, stream: TcpStream) -> Result<(), ProcError> {
        let (rx, mut tx) = stream.into_split();
        let resp = self.router.handle(rx).await.map_err(|e| { ProcError::RouteError(e) })?;
        resp.write_to(&mut tx).await.map_err(|e| { ProcError::IoError(e) })?;
        Ok(())
    }

}