use thiserror::Error;

#[derive(Error, Debug)]
pub enum HttpServerError {
    #[error("{0}")] Generic(String),
}
impl From<anyhow::Error> for HttpServerError { fn from(e: anyhow::Error) -> Self { HttpServerError::Generic(e.to_string()) } }
