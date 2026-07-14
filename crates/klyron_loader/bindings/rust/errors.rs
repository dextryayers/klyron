use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModuleResolverError {
    #[error("{0}")] Generic(String),
}
impl From<anyhow::Error> for ModuleResolverError { fn from(e: anyhow::Error) -> Self { ModuleResolverError::Generic(e.to_string()) } }
