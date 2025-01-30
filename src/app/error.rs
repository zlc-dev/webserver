use crate::http::ReqError;

#[derive(Debug)]
pub enum HandleError {
    IoError(std::io::Error),
    ReqError(ReqError),
}

#[derive(Debug)]
pub enum RouteError {
    HandleError(HandleError),
    ReqError(ReqError),
}

#[derive(Debug)]
pub enum ProcError {
    RouteError(RouteError),
    IoError(tokio::io::Error),
}