use std::{collections::HashMap, fmt::{Debug, Display}};

use tokio::io::{AsyncRead, BufReader};

use super::AsyncBufReadUtilCrlf;


#[derive(Debug, PartialEq, Eq)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
    TRACE,
    CONNECT
}

impl Method {
    pub fn from_str(s: &str) -> Option<Method> {
        match s {
            "GET" => Some(Method::GET),
            "POST" => Some(Method::POST),
            "PUT" => Some(Method::PUT),
            "DELETE" => Some(Method::DELETE),
            "PATCH" => Some(Method::PATCH),
            "HEAD" => Some(Method::HEAD),
            "OPTIONS" => Some(Method::OPTIONS),
            "TRACE" => Some(Method::TRACE),
            "CONNECT" => Some(Method::CONNECT),
            _ => None
        }
    } 
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::GET => f.write_str("GET"),
            Method::POST => f.write_str("POST"),
            Method::PUT => f.write_str("PUT"),
            Method::DELETE => f.write_str("DELETE"),
            Method::PATCH => f.write_str("PATCH"),
            Method::HEAD => f.write_str("HEAD"),
            Method::OPTIONS => f.write_str("OPTIONS"),
            Method::TRACE => f.write_str("TRACE"),
            Method::CONNECT => f.write_str("CONNECT"),
        }
    }
}

#[derive(Debug)]
pub enum ReqError {
    IOError(std::io::Error),
    FmtError,
}

pub struct HttpRequest<'a> {
    pub request_line: &'a HttpRequestLine,
    pub captures: Vec<&'a str>,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Vec<u8>>,
    pub rx: &'a mut (dyn AsyncRead + Unpin + Send), 
}

impl Debug for HttpRequest<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpRequest").field("request_line", &self.request_line).field("captures", &self.captures).field("headers", &self.headers).field("body", &self.body).finish()
    }
}

#[derive(Debug)]
pub struct HttpRequestLine {
    pub method: Method,
    pub url: String,
    pub version: String,
}

impl HttpRequestLine {

    fn new() -> Self {
        HttpRequestLine {
            method: Method::GET,
            url: String::new(),
            version: String::new(),
        }
    }

    pub async fn from_async_stream(rx: &mut (impl AsyncRead + Unpin)) -> Result<Self, ReqError> {
        
        let mut buf_reader = BufReader::new(rx);

        let mut request_line_buf = Vec::<u8>::with_capacity(1024);

        buf_reader.read_until_crlf(&mut request_line_buf).await.map_err(|e| {ReqError::IOError(e)})?;

        let request_line ;
        unsafe {
            request_line = String::from_utf8_unchecked(request_line_buf);
        }

        let mut split = request_line.split_ascii_whitespace();
        let mut req = HttpRequestLine::new();
        req.method = Method::from_str(split.next().ok_or(ReqError::FmtError)?).ok_or(ReqError::FmtError)?;
        req.url = split.next().ok_or(ReqError::FmtError)?.to_string();
        req.version = split.next().ok_or(ReqError::FmtError)?.to_string();
        
        Ok(req)
    }
}

impl<'a>  HttpRequest<'a> {
    
    pub fn create(request_line: &'a HttpRequestLine, captures: Vec<&'a str>, rx: &'a mut (impl AsyncRead + Unpin + Send)) -> HttpRequest<'a>{
        HttpRequest {
            request_line,
            captures,
            rx,
            headers: None,
            body: None,
        }
    }
}
