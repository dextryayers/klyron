use thiserror::Error;

#[derive(Error, Debug)]
pub enum WebError {
    #[error("HTTP error: {0}")] Http(String),
    #[error("URL parse error: {0}")] UrlParse(String),
    #[error("JSON error: {0}")] Json(String),
    #[error("Network error: {0}")] Network(String),
    #[error("Unsupported method: {0}")] UnsupportedMethod(String),
}
impl From<anyhow::Error> for WebError { fn from(e: anyhow::Error) -> Self { WebError::Http(e.to_string()) } }
impl From<url::ParseError> for WebError { fn from(e: url::ParseError) -> Self { WebError::UrlParse(e.to_string()) } }
impl From<serde_json::Error> for WebError { fn from(e: serde_json::Error) -> Self { WebError::Json(e.to_string()) } }
