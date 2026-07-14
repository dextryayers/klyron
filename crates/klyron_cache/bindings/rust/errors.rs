use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheManagerError {
    #[error("{0}")] Generic(String),
}
impl From<anyhow::Error> for CacheManagerError { fn from(e: anyhow::Error) -> Self { CacheManagerError::Generic(e.to_string()) } }
