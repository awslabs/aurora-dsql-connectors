// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum DsqlError {
    #[error("configuration error: {0}")]
    ConfigError(String),

    #[error("pool error: {0}")]
    PoolError(String),

    #[error("token error: {0}")]
    TokenError(String),

    #[error("connection error: {0}")]
    ConnectionError(#[source] sqlx::Error),

    #[error("database error: {0}")]
    DatabaseError(#[source] sqlx::Error),

    #[error("OCC retry exhausted after {attempts} attempts: {message}")]
    OCCRetryExhausted {
        attempts: u32,
        message: String,
        #[source]
        source: Box<DsqlError>,
    },

    #[error("{0}")]
    Error(String),
}

pub type Result<T> = std::result::Result<T, DsqlError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = DsqlError::Error("test error".to_string());
        assert_eq!(format!("{}", err), "test error");
    }

    #[test]
    fn test_occ_retry_exhausted_display() {
        let inner = sqlx::Error::Protocol("OC000".into());
        let err = DsqlError::OCCRetryExhausted {
            attempts: 3,
            message: "database error: OC000".to_string(),
            source: Box::new(DsqlError::DatabaseError(inner)),
        };
        let display = format!("{}", err);
        assert!(display.contains("3 attempts"));
        assert!(display.contains("OC000"));
    }
}
