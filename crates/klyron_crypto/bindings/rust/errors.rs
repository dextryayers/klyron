use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoProviderError {
    #[error("{0}")] Generic(String),
}
impl From<anyhow::Error> for CryptoProviderError { fn from(e: anyhow::Error) -> Self { CryptoProviderError::Generic(e.to_string()) } }
