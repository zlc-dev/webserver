use std::{collections::HashMap, io::Error};

use tokio::{fs::File, io::{AsyncReadExt, AsyncWriteExt, BufReader}};

use super::HttpVersion;

pub struct HttpResponse {
    pub version: HttpVersion,
    pub status_code: u32,
    pub status_msg: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpResponse {

    pub fn create_hello_response() -> Self {
        let mut resp = HttpResponse {
            body: "hello, world".as_bytes().to_vec(),
            ..HttpResponse::create_200_ok()
        };

        resp.headers.entry("Content-Length".to_string())
                    .and_modify(|s| { *s = resp.body.len().to_string(); })
                    .or_insert(resp.body.len().to_string());

        resp
    }

    pub fn create_200_ok() -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Length".to_string(), "0".to_string());
        HttpResponse {
            version: HttpVersion::HTTP1_1,
            status_code: 200,
            status_msg: "OK".to_string(),
            headers,
            body: Vec::new()
        }
    }

    pub fn create_404_not_found() -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Length".to_string(), "0".to_string());
        HttpResponse {
            version: HttpVersion::HTTP1_1,
            status_code: 404,
            status_msg: "Not Found".to_string(),
            headers,
            body: Vec::new(),
        }
    }

    pub fn create_405_method_not_allowed() -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Length".to_string(), "0".to_string());
        HttpResponse {
            version: HttpVersion::HTTP1_1,
            status_code: 405,
            status_msg: "Method Not Allowed".to_string(),
            headers,
            body: Vec::new()
        }
    }

    pub fn create_500_internal_server_error() -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Length".to_string(), "0".to_string());
        HttpResponse {
            version: HttpVersion::HTTP1_1,
            status_code: 500,
            status_msg: "Internal Server Error".to_string(),
            headers,
            body: Vec::new()
        }
    }

    pub async fn from_html_file(path: &'static str) -> Self {
        
        if let Ok(html_file) = File::open(path).await {
            let mut buf = Vec::<u8>::new();
            let mut buf_reader = BufReader::new(html_file);
            loop {
                if let Ok(s) = buf_reader.read_buf(&mut buf).await {
                    if s == 0 {
                        let mut resp = HttpResponse {
                            body: buf,
                            ..Self::create_200_ok()
                        };
                        resp.headers.entry("Content-Length".to_string())
                                    .and_modify(|s| { *s = resp.body.len().to_string(); })
                                    .or_insert(resp.body.len().to_string());
                        return resp;
                    }
                } else {
                    return Self::create_500_internal_server_error()
                }
            }
        } else {
            Self::create_500_internal_server_error()
        }
    }

    pub async fn write_to(&self, tx: &mut (impl AsyncWriteExt + Unpin)) -> Result<(), Error> {

        tx.write_all(self.version.to_str().as_bytes()).await?;
        tx.write_u8(b' ').await?;
        tx.write_all(self.status_code.to_string().as_bytes()).await?;
        tx.write_u8(b' ').await?;
        tx.write_all(self.status_msg.as_bytes()).await?;
        tx.write_all(b"\r\n").await?;
        for (k, v) in &self.headers {
            tx.write_all(k.as_bytes()).await?;
            tx.write_all(b": ").await?;
            tx.write_all(v.as_bytes()).await?;
            tx.write_all(b"\r\n").await?;
        }
        tx.write_all(b"\r\n").await?;
        tx.write_all(&self.body[..]).await?;
        tx.write_all(b"\r\n").await?;
        tx.flush().await?;

        Ok(())
    }

}