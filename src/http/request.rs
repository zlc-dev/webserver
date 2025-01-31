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

    pub fn to_str(&self) -> &'static str {
        match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::DELETE => "DELETE",
            Method::PATCH => "PATCH",
            Method::HEAD => "HEAD",
            Method::OPTIONS => "OPTIONS",
            Method::TRACE => "TRACE",
            Method::CONNECT => "CONNECT",
        }
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_str())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum HttpVersion {
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

    pub fn to_str(&self) -> &'static str {
        match self {
            HttpVersion::HTTP1_0 => "HTTP/1.0",
            HttpVersion::HTTP1_1 => "HTTP/1.1",
        }
    }

}

impl Display for HttpVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_str())
    }
}

#[derive(Debug)]
pub enum ReqError {
    IOError(std::io::Error),
    EmptyReq,
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

        if 0 == buf_reader.read_until_crlf(&mut request_line_buf).await.map_err(|e| {ReqError::IOError(e)})? {
            return Err(ReqError::EmptyReq);
        }

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
