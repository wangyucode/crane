use std::{env, net::SocketAddr};

use actix_web::{get, http::StatusCode, App, HttpResponse, HttpServer, Responder};

#[get("/")]
async fn greet() -> impl Responder {
    HttpResponse::NotFound().body("Crane runs ok, use /deploy endpoint")
}

#[get("/deploy")]
async fn deploy() -> impl Responder {
    HttpResponse::Ok()
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
    println!("Listening on port {}", addr);
    HttpServer::new(|| {
        App::new()
            .service(greet)
            .service(deploy)
            .service(cancel)
    })
    .bind(addr)?
    .run()
    .await
}