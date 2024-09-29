use std::collections::HashMap;

use http_body_util::Full;
use hyper::{body::Bytes, Response, StatusCode};
use url::form_urlencoded;

/// return a response with the given code and body
/// # Arguments status: Option<StatusCode>
/// # Arguments body: Option<&str>
/// # Returns Response<Full<Bytes>>
pub fn get_response(
    status: Option<StatusCode>,
    body: Option<&'static str>,
) -> Response<Full<Bytes>> {
    let status = status.unwrap_or(StatusCode::OK);
    let body_str = body.unwrap_or(status.canonical_reason().unwrap_or("Ok"));
    if !status.is_success() {
        eprintln!("{}", body_str)
    }

    Response::builder()
        .header("Server", "Crane")
        .status(status)
        .body(Full::new(Bytes::from(body_str)))
        .unwrap()
}


// fn extract_query_param(query_string: &str, param_name: str) -> Option<&'a str> {
//     let mut params: HashMap<&str, &str> = HashMap::new();
//     for (key, value) in form_urlencoded::parse(query_string.as_bytes()) {
//         params.insert(&key, &value);
//     }
//     params.get(param_name)
// }


