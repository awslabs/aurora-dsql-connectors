use thiserror::Error;

#[derive(Error, Debug)]
pub enum DsqlError {
    #[error("configuration error: {0}")]
    ConfigError(String),

    #[error("token error: {0}")]
    TokenError(String),

    #[error("connection error: {0}")]
    ConnectionError(String),

    #[error("database error: {0}")]
    DatabaseError(String),

    #[error("{0}")]
    Error(String),
}

pub type Result<T> = std::result::Result<T, DsqlError>;
