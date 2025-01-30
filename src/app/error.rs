use crate::http::ReqError;

#[derive(Debug)]
pub enum HandleError {
    IoError(std::io::Error),
    ReqError(ReqError),
}

#[derive(Debug)]
pub enum RouteError {
    HandleError(HandleError),
    NotFound,
}

#[derive(Debug)]
pub enum ProcError {
    ReqError(ReqError),
    RouteError(RouteError),
    IoError(tokio::io::Error),
}