// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

//! bb8 connection pool integration for Aurora DSQL.
//!
//! This crate provides a [`bb8::ManageConnection`] implementation that
//! generates a fresh IAM auth token for each new connection. Use this
//! when you prefer bb8 over sqlx's built-in pool.

use aurora_dsql_sqlx_connector::{DsqlConnectOptions, DsqlError};
use sqlx::{Connection, PgConnection};

/// Manages bb8 pool connections to Aurora DSQL, generating a fresh
/// IAM auth token for every new connection.
pub struct DsqlConnectionManager {
    config: DsqlConnectOptions,
}

impl DsqlConnectionManager {
    /// Creates a new connection manager with the given options.
    pub fn new(config: DsqlConnectOptions) -> Self {
        Self { config }
    }
}

impl bb8::ManageConnection for DsqlConnectionManager {
    type Connection = PgConnection;
    type Error = DsqlError;

    async fn connect(&self) -> Result<PgConnection, DsqlError> {
        aurora_dsql_sqlx_connector::connection::connect_with(&self.config).await
    }

    async fn is_valid(&self, conn: &mut PgConnection) -> Result<(), DsqlError> {
        conn.ping().await.map_err(DsqlError::ConnectionError)
    }

    /// Always returns `false`; connection health is checked via `is_valid` (ping).
    fn has_broken(&self, _conn: &mut PgConnection) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_dsql_sqlx_connector::DsqlConnectOptionsBuilder;
    use sqlx::postgres::PgConnectOptions;

    #[test]
    fn test_manager_creation() {
        let pg = PgConnectOptions::new()
            .host("example.dsql.us-east-1.on.aws")
            .username("admin")
            .database("postgres");

        let config = DsqlConnectOptionsBuilder::default()
            .pg_connect_options(pg)
            .build()
            .unwrap();

        let _manager = DsqlConnectionManager::new(config);
    }

    #[test]
    fn test_implements_manage_connection() {
        fn assert_manage_connection<T: bb8::ManageConnection>() {}
        assert_manage_connection::<DsqlConnectionManager>();
    }

    #[tokio::test]
    async fn test_manager_connect_fails_without_database() {
        use bb8::ManageConnection;

        let config = DsqlConnectOptions::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres",
        )
        .unwrap();

        let manager = DsqlConnectionManager::new(config);

        // Will fail — no real database or credentials
        let result = manager.connect().await;
        assert!(result.is_err());
    }
}
