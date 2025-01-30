use std::{collections::HashMap, fmt::{Debug, Display}};

use tokio::io::{AsyncRead, AsyncBufRead};

use super::AsyncBufReadUtilCrlf;


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
enum HttpVersion {
    HTTP1_0,
    HTTP1_1,
}

impl HttpVersion {
    pub fn from_str(s: &str) -> Option<HttpVersion> {
        match s {
            "HTTP/1.0" => Some(HttpVersion::HTTP1_0),
            "HTTP/1.1" => Some(HttpVersion::HTTP1_1),
            _ => None
        }
    } 
}

#[derive(Debug)]
pub enum ReqError {
    IOError(std::io::Error),
    FmtError,
}

#[derive(Debug)]
pub struct HttpRequestHeader {
    pub method: Method,
    pub url: String,
    pub version: HttpVersion,
    pub paras: HashMap<String, String>,
}

impl HttpRequestHeader {
    pub fn new() -> Self {
        HttpRequestHeader {
            method: Method::GET,
            url: String::new(),
            version: HttpVersion::HTTP1_1,
            paras: HashMap::new(),
        }
    }

    pub async fn from_async_stream(buf_reader: &'_ mut (impl AsyncBufRead + Unpin + Send)) -> Result<Self, ReqError> {
    
        let mut request_line_buf = Vec::<u8>::with_capacity(1024);

        buf_reader.read_until_crlf(&mut request_line_buf).await.map_err(|e| {ReqError::IOError(e)})?;

        let request_line ;
        unsafe {
            request_line = String::from_utf8_unchecked(request_line_buf);
        }

        let mut split = request_line.split_ascii_whitespace();
        let mut req = HttpRequestHeader::new();
        req.method = Method::from_str(split.next().ok_or(ReqError::FmtError)?).ok_or(ReqError::FmtError)?;
        req.url = split.next().ok_or(ReqError::FmtError)?.to_string();
        req.version = HttpVersion::from_str(split.next().ok_or(ReqError::FmtError)?).ok_or(ReqError::FmtError)?;

        loop {
            let mut header_buf = Vec::<u8>::new();
            let s = buf_reader.read_until_crlf(&mut header_buf).await.map_err(|e| { ReqError::IOError(e) })?;
            if s == 2 {
                break;
            }
            let header_line ;
            unsafe {
                header_line = String::from_utf8_unchecked(header_buf);
            }
            let mut split = header_line.splitn(2, ':');
            let k = split.next().ok_or(ReqError::FmtError)?;
            let v = split.next().ok_or(ReqError::FmtError)?;
            req.paras.insert(
                k.to_string(),
                v.trim().to_string()
            );
        }
        
        Ok(req)
    }



}

pub struct HttpRequest<'a> {
    pub header: &'a HttpRequestHeader,
    url_paras: Option<Vec<&'a str>>,
    body: Option<Vec<u8>>,
    buf_reader: &'a mut (dyn AsyncBufRead + Unpin + Send), 
}

unsafe impl<'a> Sync for HttpRequest<'a> {
    
}

impl Debug for HttpRequest<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpRequest").field("header", &self.header).field("url_paras", &self.url_paras).field("body", &self.body).finish()
    }
}


impl<'a>  HttpRequest<'a> {
    
    pub fn new(header:&'a HttpRequestHeader ,rx: &'a mut (impl AsyncBufRead + Unpin + Send)) -> HttpRequest<'a>{
        HttpRequest {
            header,
            url_paras: None,
            buf_reader: rx,
            body: None,
        }
    }
}
