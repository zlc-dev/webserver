use std::{collections::HashMap, io::Error};

use tokio::{fs::File, io::{AsyncReadExt, AsyncWriteExt, BufReader}};

pub struct HttpResponse {
    pub version: String,
    pub status_code: u32,
    pub status_msg: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpResponse {

    pub fn create_hello_response() -> Self {
        HttpResponse {
            version: "HTTP/1.1".to_string(),
            status_code: 200,
            status_msg: "OK".to_string(),
            headers: HashMap::new(),
            body: "hello, world".as_bytes().to_vec()
        }
    }

    pub fn create_200_ok() -> Self {
        HttpResponse {
            version: "HTTP/1.1".to_string(),
            status_code: 200,
            status_msg: "OK".to_string(),
            headers: HashMap::new(),
            body: Vec::new()
        }
    }

    pub fn create_500_internal_server_error() -> Self {
        HttpResponse {
            version: "HTTP/1.1".to_string(),
            status_code: 500,
            status_msg: "Internal Server Error".to_string(),
            headers: HashMap::new(),
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
                        return HttpResponse {
                            body: buf,
                            ..Self::create_200_ok()
                        }
                    }
                } else {
                    return Self::create_500_internal_server_error()
                }
            }
        } else {
            Self::create_500_internal_server_error()
        }
    }

    pub fn create_404_not_found() -> Self {
        HttpResponse {
            version: "HTTP/1.1".to_string(),
            status_code: 404,
            status_msg: "NotFound".to_string(),
            headers: HashMap::new(),
            body: b"<h1>Not Found</h1>".to_vec()
        }
    }



    pub async fn write_to(&self, tx: &mut (impl AsyncWriteExt + Unpin)) -> Result<(), Error> {

        tx.write_all(self.version.as_bytes()).await?;
        tx.write_all(self.status_code.to_string().as_bytes()).await?;
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

        Ok(())
    }

}