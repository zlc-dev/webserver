use tokio::{io::BufReader, net::TcpStream};
use crate::http::{HttpRequest, HttpRequestHeader};

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
        let mut buf_reader = BufReader::new(rx);
        let req_header = HttpRequestHeader::from_async_stream(&mut buf_reader).await.map_err(|e| { ProcError::ReqError(e)})?;
        let resp = self.router.routing(&mut HttpRequest::new(&req_header, &mut buf_reader)).await.map_err(|e| { ProcError::RouteError(e) })?;
        resp.write_to(&mut tx).await.map_err(|e| { ProcError::IoError(e) })?;
        Ok(())
    }

}