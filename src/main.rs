mod processor;
mod http;
mod app;

use std::{future::Future, pin::Pin, sync::Arc};

use app::Application;

use http::{HttpRequest, HttpResponse};
use processor::HttpResult;

async fn hello(req: HttpRequest<'_>) -> HttpResult {
    println!("{:?}", &req);
    Ok(HttpResponse::from_html_file("./asset/hello.html").await)
}

async fn notfound(req: HttpRequest<'_>) -> HttpResult {
    println!("{:?}", &req);
    Ok(HttpResponse{status_code:404, status_msg: "Not Found".to_string(), ..HttpResponse::from_html_file("./asset/notfound.html").await})
}

#[tokio::main]
async fn main()  {
    let app = Application::new();
    app.listen_tcp("127.0.0.1:8000").await.unwrap()
        .register("/hello", Arc::new(boxed_f!(hello))).unwrap()
        .register("/hello/{}", Arc::new(boxed_f!(hello))).unwrap()
        .register("{a}", Arc::new(boxed_f!(notfound))).unwrap()
        .run().await;
}
