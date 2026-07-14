use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessManagerError {
    #[error("{0}")] Generic(String),
}
impl From<anyhow::Error> for ProcessManagerError { fn from(e: anyhow::Error) -> Self { ProcessManagerError::Generic(e.to_string()) } }
