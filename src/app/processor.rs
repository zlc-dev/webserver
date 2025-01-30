use tokio::{io::BufReader, net::TcpStream};
use super::{ProcError, Router};

pub struct Processor {
    pub router: Router
}

impl Processor{

    pub fn new() -> Self {
        Processor {
            router: Router::new(),
        }
    }

    pub async fn handle(&self, mut stream: TcpStream) -> Result<(), ProcError> {
        let (rx, mut tx) = stream.split();
        let resp = self.router.handle(BufReader::new(rx)).await.map_err(|e| { ProcError::RouteError(e) })?;
        resp.write_to(&mut tx).await.map_err(|e| { ProcError::IoError(e) })?;
        Ok(())
    }

}