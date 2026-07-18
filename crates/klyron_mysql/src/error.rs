use thiserror::Error;

#[derive(Error, Debug)]
pub enum MySqlError {
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("no rows returned")]
    NoRows,

    #[error("migration error: {0}")]
    Migration(String),
}

pub type Result<T> = std::result::Result<T, MySqlError>;
