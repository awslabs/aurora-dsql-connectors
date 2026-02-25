use thiserror::Error;

#[derive(Error, Debug)]
pub enum DsqlError {
    #[error("{0}")]
    Error(String),
}

pub type Result<T> = std::result::Result<T, DsqlError>;
