mod util;

use std::convert::Infallible;
use std::{env, iter};
use std::fmt::format;
use std::net::SocketAddr;

use http_body_util::combinators::{BoxBody, MapFrame};
use http_body_util::{BodyExt, StreamBody};
use hyper::body::{Body, Bytes, Frame, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Error, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use util::{extract_query_param, get_response};



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let port = env::var("PORT").unwrap_or("8594".to_string());
    let addr = SocketAddr::from(([0, 0, 0, 0], port.parse().unwrap()));
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on port {}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(handler))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn handler(req: Request<Incoming>) -> Result<Response<BoxBody<Bytes, Infallible>>, Error> {
    // println!("Received request{:?}", req);
    if req.uri().path() != "/" {
        return Ok(get_response(Some(StatusCode::NOT_FOUND), None));
    }
    // get the download url from the request
    let query = req.uri().query();
    if query.is_none() {
        return Ok(get_response(Some(StatusCode::BAD_REQUEST), Some("url must be provide in query!"))); 
    }
    let url = extract_query_param(query.unwrap(), "url");
    if url.is_none() {
        return Ok(get_response(Some(StatusCode::BAD_REQUEST), Some("url must be provide in query!")));
    }

    let url = url.unwrap();
    println!("Download URL: {}", url);

    if !url.ends_with(".tar.gz") {
        return Ok(get_response(Some(StatusCode::BAD_REQUEST), Some("url must be a tar.gz file!")));
    }
    
    // Deployer::new(url, body).run().await;
    
    // Ok(Response::new(StreamBody::new().boxed()))
    Ok(get_response(None, None))
}


