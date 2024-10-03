use std::{env, net::SocketAddr, time::{SystemTime, UNIX_EPOCH}};

use actix_web::{
    get,
    http::StatusCode,
    web::{self, Query},
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
mod deployer;

#[get("/")]
async fn greet() -> impl Responder {
    HttpResponse::NotFound().body("Crane runs ok, use /deploy endpoint")
}

#[derive(Debug, Deserialize)]
pub struct DeployQuery {
    url: String,
}

#[get("/deploy")]
async fn deploy(req: HttpRequest) -> impl Responder {
    let key = req.headers().get("X-Api-Key");
    if key.is_none() {
        return HttpResponse::Unauthorized().body("Missing X-Api-Key header");
    }
    let key = key.unwrap().as_bytes();
    if key != env::var("API_KEY").unwrap().as_bytes() {
        return HttpResponse::Forbidden().body("Invalid X-Api-Key header");
    }
    let url = Query::<DeployQuery>::from_query(req.query_string());
    if url.is_err() {
        return HttpResponse::BadRequest().body("Missing url query parameter");
    }
    let url = url.unwrap().url.clone();
    if !url.ends_with(".tar.gz") {
        return HttpResponse::BadRequest().body("currently only .tar.gz files are supported");
    }

    let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
    println!("deploying triggered, url: {}, date: {}", url, timestamp);
    let (tx, rx) = mpsc::channel(1);
    let stream =
        ReceiverStream::new(rx).map(|msg: String| Ok::<web::Bytes, Error>(web::Bytes::from(msg)));

    tokio::spawn(async move {
        let result = deployer::deploy(&tx, url).await;
        println!("deploy result: {result:?}");
        if result.is_err() {
            tx.send(format!("Deployment failed: {result:?}"))
                .await
                .unwrap();
        }
    });

    HttpResponse::Ok()
        .content_type("text/plain")
        .append_header(("Transfer-Encoding", "chunked"))
        .streaming(stream)
}

#[get("/cancel")]
async fn cancel() -> impl Responder {
    // TODO: cancel deployment
    HttpResponse::NotImplemented().body(StatusCode::NOT_IMPLEMENTED.canonical_reason().unwrap())
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    env::var("API_KEY").expect("API_KEY must be set");
    let addr = SocketAddr::from(([0, 0, 0, 0], 8594));
    println!("Listening on port {addr}");
    HttpServer::new(|| App::new().service(greet).service(deploy).service(cancel))
        .bind(addr)?
        .run()
        .await
}

#[cfg(test)]
mod tests {

    use super::*;
    use actix_web::{test, App};
    #[actix_web::test]
    async fn test_greet() {
        let app = test::init_service(App::new().service(greet)).await;
        let req = test::TestRequest::get().to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
        let body = test::read_body(res).await;
        assert_eq!(
            body, "Crane runs ok, use /deploy endpoint",
            "test great body"
        );
    }

    #[actix_web::test]
    async fn test_deploy() {
        let app = test::init_service(App::new().service(deploy)).await;
        let req = test::TestRequest::get().uri("/deploy").to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(
            res.status(),
            StatusCode::UNAUTHORIZED,
            "test deploy if API_KEY is missing"
        );
        // set the API_KEY to right
        env::set_var("API_KEY", "right");

        let req = test::TestRequest::get()
            .uri("/deploy")
            .append_header(("X-Api-Key", "wrong"))
            .to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(
            res.status(),
            StatusCode::FORBIDDEN,
            "test deploy if API_KEY is wrong"
        );
        
        env::set_var("API_KEY", "right");
        let req = test::TestRequest::get()
            .uri("/deploy")
            .append_header(("X-Api-Key", "right"))
            .to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(
            res.status(),
            StatusCode::BAD_REQUEST,
            "test status if url is missing"
        );
        let body = test::read_body(res).await;
        assert_eq!(
            body, "Missing url query parameter",
            "test body if url is missing"
        );
        // test status if url is not end with .tar.gz
        let req = test::TestRequest::get()
            .uri("/deploy?url=https://example.com")
            .append_header(("X-Api-Key", "right"))
            .to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(
            res.status(),
            StatusCode::BAD_REQUEST,
            "test status if url is not end with .tar.gz"
        );
        let body = test::read_body(res).await;
        assert_eq!(
            body, "currently only .tar.gz files are supported",
            "test body if url is not end with .tar.gz"
        );
        // test status if url is end with .tar.gz;
        let req = test::TestRequest::get()
            .uri("/deploy?url=https://example.com/test.tar.gz")
            .append_header(("X-Api-Key", "right"))
            .to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(
            res.status(),
            StatusCode::OK,
            "test status if url is end with .tar.gz"
        );
        let body = test::read_body(res).await;
        assert!(body.starts_with("start deploy https://example.com/test.tar.gz\r\n".as_bytes()));
    }
}
