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
    MethodNotAllowed,
}

#[derive(Debug)]
pub enum ProcError {
    ReqError(ReqError),
    HandleError(HandleError),
    IoError(std::io::Error),
}