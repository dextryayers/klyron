use thiserror::Error;

#[derive(Error, Debug)]
pub enum DnsResolverError {
    #[error("{0}")] Generic(String),
}
impl From<anyhow::Error> for DnsResolverError { fn from(e: anyhow::Error) -> Self { DnsResolverError::Generic(e.to_string()) } }
