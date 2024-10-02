mod deployer;
mod util;

use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use deployer::Deployer;
use futures::stream::StreamExt;
use http_body_util::combinators::BoxBody;
use http_body_util::StreamBody;
use hyper::body::{Bytes, Frame, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Error, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use util::{extract_query_param, get_response};

#[get("/")]
async fn greet() -> impl Responder {
    HttpResponse::NotFound().body("Crane runs ok, use /deploy endpoint")
}

#[get("/deploy")]
async fn deploy() -> impl Responder {
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::var("API_KEY").expect("API_KEY must be set");
    // let addr = SocketAddr::from(([0, 0, 0, 0], 8594));
    // let listener = TcpListener::bind(addr).await?;
    // println!("Listening on port {}", addr);

    // loop {
    //     let (stream, _) = listener.accept().await?;
    //     let io = TokioIo::new(stream);
    //     tokio::task::spawn(async move {
    //         if let Err(err) = http1::Builder::new()
    //             // `service_fn` converts our function in a `Service`
    //             .serve_connection(io, service_fn(handler))
    //             .await
    //         {
    //             eprintln!("Error serving connection: {:?}", err);
    //         }
    //     });
    // }
    let addr = SocketAddr::from(([0, 0, 0, 0], 8594));
    let server = HttpServer::new(|| {App::new().service(greet).service(deploy)})
        .bind(addr).unwrap()?
        .run()
        .await;
}

async fn handler(req: Request<Incoming>) -> Result<Response<BoxBody<Bytes, Infallible>>, Error> {
    // println!("Received request{:?}", req);
    if req.uri().path() != "/" {
        return Ok(get_response(Some(StatusCode::NOT_FOUND), None));
    }
    let key = req.headers().get("X-Api-Key");
    if key.is_none() {
        return Ok(get_response(
            Some(StatusCode::UNAUTHORIZED),
            Some("X-Api-Key is required!"),
        ));
    }
    let key = key.unwrap();
    match key.to_str().unwrap() != env::var("API_KEY").unwrap() {
        true => {
            return Ok(get_response(
                Some(StatusCode::UNAUTHORIZED),
                Some("Invalid API key!"),
            ));
        }
        false => (),
    }
    // get the download url from the request
    let query = req.uri().query();
    if query.is_none() {
        return Ok(get_response(
            Some(StatusCode::BAD_REQUEST),
            Some("url must be provide in query!"),
        ));
    }
    let query = query.unwrap();
    let url = extract_query_param(query, "url");
    if url.is_none() {
        return Ok(get_response(
            Some(StatusCode::BAD_REQUEST),
            Some("url must be provide in query!"),
        ));
    }

    let url = url.unwrap();
    println!("Download URL: {}", url);

    if !url.ends_with(".tar.gz") {
        return Ok(get_response(
            Some(StatusCode::BAD_REQUEST),
            Some("url must be a tar.gz file!"),
        ));
    }

    // Create a channel to send progress updates
    let (tx, rx) = mpsc::channel(2);
    let stream =
        ReceiverStream::new(rx).map(|frame| Ok(Frame::data(Bytes::from(format!("{}", frame)))));

    tokio::spawn(async move {
        let deployer = Deployer::new(&tx);
        if let Err(e) = deployer.download(&url).await {
            let error = format!("Download error: {:?}", e);
            eprintln!("{}", error);
            if let Err(wtf) = tx.send(error).await {
                eprintln!("{}", wtf);
            };
        }
        if let Err(e) = deployer.deploy().await {
            let error = format!("Deploy error: {:?}", e);
            eprintln!("{}", error);
            if let Err(wtf) = tx.send(error).await {
                eprintln!("{}", wtf);
            }
        }
    });

    let body = BoxBody::new(StreamBody::new(stream));

    let response = Response::builder()
        .header("Server", "Crane")
        .header("Content-Type", "text/plain; charset=utf-8")
        .header("Transfer-Encoding", "chunked")
        .body(body)
        .unwrap();

    Ok(response)
}
