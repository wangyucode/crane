use std::convert::Infallible;

use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::{body::Bytes, Response, StatusCode};
use url::form_urlencoded;

/// return a response with the given code and body
/// # Arguments status: Option<StatusCode>
/// # Arguments body: Option<&str>
/// # Returns Response<Full<Bytes>>
pub fn get_response(
    status: Option<StatusCode>,
    body: Option<&'static str>,
) -> Response<BoxBody<Bytes, Infallible>> {
    let status = status.unwrap_or(StatusCode::OK);
    let body_str = body.unwrap_or(status.canonical_reason().unwrap_or("Ok"));
    if !status.is_success() {
        eprintln!("{}", body_str)
    }

    Response::builder()
        .header("Server", "Crane")
        .status(status)
        .body(Full::new(Bytes::from(body_str)).boxed())
        .unwrap()
}

/// extract a query param from the given query string
/// # Arguments query_string: &str
/// # Arguments param_name: &str
/// # Returns Option<String>
pub fn extract_query_param(query_string: &str, param_name: &str) -> Option<String> {
    for (key, value) in form_urlencoded::parse(query_string.as_bytes()) {
        if key == param_name {
            return Some(value.into_owned());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_query_param() {
        let encoded: String = form_urlencoded::Serializer::new(String::new())
            .append_pair("url", "https://wycode.cn/download/dist.tar.gz")
            .append_pair("path", "/app/data/")
            .finish();
        assert_eq!(encoded, "url=https%3A%2F%2Fwycode.cn%2Fdownload%2Fdist.tar.gz&path=%2Fapp%2Fdata%2F");

        let url = extract_query_param(&encoded, "url");
        assert_eq!(url.unwrap(), "https://wycode.cn/download/dist.tar.gz");
        
        let name = extract_query_param(&encoded, "path");
        assert_eq!(name.unwrap(), "/app/data/");
    }
}
