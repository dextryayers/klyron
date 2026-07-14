use thiserror::Error;

#[derive(Error, Debug)]
pub enum NodeGlobalsError {
    #[error("{0}")] Generic(String),
}
impl From<anyhow::Error> for NodeGlobalsError { fn from(e: anyhow::Error) -> Self { NodeGlobalsError::Generic(e.to_string()) } }
