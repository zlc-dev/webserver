mod app;
mod http;

use std::{future::Future, pin::Pin, sync::Arc};
use http::{HttpRequest, HttpResponse};
use app::{HandleError, HttpResult, Application};

async fn hello(req: &mut HttpRequest<'_>, captures: Vec<&'_ str>) -> HttpResult {
    println!("{:?}", req.header.paras.get("Connection"));
    Ok(HttpResponse::from_html_file("./asset/hello.html").await)
}

async fn notfound(req: &mut HttpRequest<'_>, captures: Vec<&'_ str>) -> HttpResult {
    // println!("{:?}", &req);
    Ok(HttpResponse{status_code:404, status_msg: "Not Found".to_string(), ..HttpResponse::from_html_file("./asset/notfound.html").await})
}

#[tokio::main]
async fn main()  {

    let app = Application::new();
    app.listen_tcp("127.0.0.1:8000").await.unwrap()
        .listen_tcp("127.0.0.1:8080").await.unwrap()
        .registrar()
        .get()
        .at("/hello")
        .register("", ep_wrap!(hello))
        .register("/{}", ep_wrap!(hello))
        .app()
        .run().await;
}
