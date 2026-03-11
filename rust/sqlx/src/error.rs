// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

pub type Result<T> = std::result::Result<T, DsqlError>;

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

    #[error("OCC retry exhausted after {attempts} attempts: {source}")]
    OCCRetryExhausted {
        attempts: u32,
        #[source]
        source: Box<DsqlError>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_occ_retry_exhausted_display() {
        let inner = sqlx::Error::Protocol("OC000".into());
        let err = DsqlError::OCCRetryExhausted {
            attempts: 3,
            source: Box::new(DsqlError::DatabaseError(inner)),
        };
        let display = format!("{}", err);
        assert!(display.contains("3 attempts"));
        assert!(display.contains("OC000"));
    }

    #[test]
    fn test_config_error_display() {
        let err = DsqlError::ConfigError("missing host".into());
        assert_eq!(err.to_string(), "configuration error: missing host");
    }

    #[test]
    fn test_pool_error_display() {
        let err = DsqlError::PoolError("connection pool timed out".into());
        assert_eq!(err.to_string(), "pool error: connection pool timed out");
    }

    #[test]
    fn test_token_error_display() {
        let err = DsqlError::TokenError("No credentials provider found".into());
        assert_eq!(
            err.to_string(),
            "token error: No credentials provider found"
        );
    }

    #[test]
    fn test_connection_error_display() {
        let inner = sqlx::Error::Protocol("connection refused".into());
        let err = DsqlError::ConnectionError(inner);
        assert!(err.to_string().contains("connection error"));
        assert!(err.to_string().contains("connection refused"));
    }

    #[test]
    fn test_database_error_display() {
        let inner = sqlx::Error::Protocol("query failed".into());
        let err = DsqlError::DatabaseError(inner);
        assert!(err.to_string().contains("database error"));
        assert!(err.to_string().contains("query failed"));
    }

    #[test]
    fn test_connection_error_source() {
        let inner = sqlx::Error::Protocol("connection refused".into());
        let err = DsqlError::ConnectionError(inner);
        let std_err: &dyn std::error::Error = &err;
        assert!(std_err.source().is_some());
    }

    #[test]
    fn test_database_error_source() {
        let inner = sqlx::Error::Protocol("query failed".into());
        let err = DsqlError::DatabaseError(inner);
        let std_err: &dyn std::error::Error = &err;
        assert!(std_err.source().is_some());
    }
}
