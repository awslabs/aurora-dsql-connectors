// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::{DsqlConfig, Result};
use sqlx::PgConnection;

/// Connect to Aurora DSQL from a connection string, returning a native `PgConnection`.
///
/// Generates a fresh IAM token at connect time. IAM tokens are valid for 15 minutes.
///
/// For production workloads, use `DsqlPool` which provides automatic token refresh
/// and connection pooling.
pub async fn dsql_connect(url: &str) -> Result<PgConnection> {
    let config = DsqlConfig::from_connection_string(url).await?;
    config.connect().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DsqlError;

    #[tokio::test]
    async fn test_dsql_connect_invalid_url() {
        let result = dsql_connect("not-a-url").await;
        assert!(result.is_err(), "Should fail with invalid URL");
    }

    #[tokio::test]
    async fn test_dsql_connect_invalid_scheme() {
        let result = dsql_connect("mysql://admin@example.dsql.us-east-1.on.aws/postgres").await;
        assert!(result.is_err(), "Should fail with non-postgres scheme");
    }

    #[tokio::test]
    async fn test_dsql_connect_empty_string() {
        let result = dsql_connect("").await;
        assert!(result.is_err(), "Should fail with empty connection string");
    }

    #[tokio::test]
    async fn test_dsql_connect_unreachable_host() {
        let result = dsql_connect("postgres://admin@example.dsql.us-east-1.on.aws/postgres").await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        match &err {
            DsqlError::TokenError(_) | DsqlError::ConnectionError(_) => {}
            other => panic!("Expected TokenError or ConnectionError, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_connect_from_config() {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres",
        )
        .await
        .unwrap();

        let result = config.connect().await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        match &err {
            DsqlError::TokenError(_) | DsqlError::ConnectionError(_) => {}
            other => panic!("Expected TokenError or ConnectionError, got: {:?}", other),
        }
    }
}
