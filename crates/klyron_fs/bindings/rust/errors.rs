use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileSystemError {
    #[error("{0}")] Generic(String),
}
impl From<anyhow::Error> for FileSystemError { fn from(e: anyhow::Error) -> Self { FileSystemError::Generic(e.to_string()) } }
