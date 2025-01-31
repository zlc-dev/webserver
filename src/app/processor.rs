use tokio::{io::{AsyncBufRead, AsyncWrite, BufReader}, net::tcp::{ReadHalf, WriteHalf}};
use crate::http::{HttpRequest, HttpRequestHeader, HttpResponse, ReqError};

use super::{ProcError, RouteError, Router};

pub enum ConnectionState {
    Opening,
    Closed
}

pub struct ProcRes {
    pub connect_state: ConnectionState
}

pub struct Processor {
    pub router: Router,
}

impl Processor{

    pub fn new() -> Self {
        Processor {
            router: Router::new(),
        }
    }

    pub async fn handle(&self, buf_rx: &mut (impl AsyncBufRead + Send + Unpin), tx: &mut (impl AsyncWrite + Unpin)) -> Result<ProcRes, ProcError> {
        let req_header = match HttpRequestHeader::from_async_stream(buf_rx).await {
            Err(ReqError::EmptyReq) => {
                return Ok(ProcRes {connect_state: ConnectionState::Closed});
            },
            Err(e) => {
                return Err(ProcError::ReqError(e));
            },
            Ok(req_header) => req_header,
        };
        let mut req =  HttpRequest::new(&req_header, buf_rx);
        let mut resp = match self.router.routing(&mut req).await {
            Ok(resp) => resp,
            Err(RouteError::MethodNotAllowed) => {
                HttpResponse::create_405_method_not_allowed()
            },
            Err(RouteError::NotFound) => {
                HttpResponse::create_404_not_found()
            },
            Err(RouteError::HandleError(e)) => {
                return Err(ProcError::HandleError(e));
            }
        };
        resp.version = req.header.version;
        resp.write_to(tx).await.map_err(|e| { ProcError::IoError(e) })?;
        let mut ret  = ProcRes {connect_state: ConnectionState::Opening};
        match req.header.version {
            crate::http::HttpVersion::HTTP1_0 => {
                if req.header.paras.get("Connection") == Some(&"Keep-Alive".to_string()) {
                    ret.connect_state = ConnectionState::Opening;
                } else {
                    ret.connect_state = ConnectionState::Closed;
                }
            },
            crate::http::HttpVersion::HTTP1_1 => {
                if req.header.paras.get("Connection") == Some(&"close".to_string()) {
                    ret.connect_state = ConnectionState::Closed;
                } else {
                    ret.connect_state = ConnectionState::Opening;
                }
            },
        }
        Ok(ret)
    }

}