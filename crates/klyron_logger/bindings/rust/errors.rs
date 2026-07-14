use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoggerError {
    #[error("{0}")] Generic(String),
}
impl From<anyhow::Error> for LoggerError { fn from(e: anyhow::Error) -> Self { LoggerError::Generic(e.to_string()) } }
