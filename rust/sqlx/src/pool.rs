// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use crate::token;
use crate::util::Region;
use crate::{DsqlConfig, DsqlError, Result};
use sqlx::{Connection, PgConnection};

// We use bb8 instead of sqlx::Pool because sqlx's built-in pool bakes the
// password into PgConnectOptions at pool creation time with no hook to refresh
// it per-connection (see launchbadge/sqlx#3117).
// bb8's ManageConnection::connect() lets us generate a fresh IAM token for
// each new connection, which is the recommended pattern. Note token generation
// is a local SigV4 presigning operation with negligible cost.

pub struct DsqlConnectionManager {
    config: DsqlConfig,
    region: Region,
    sdk_config: aws_config::SdkConfig,
}

impl bb8::ManageConnection for DsqlConnectionManager {
    type Connection = PgConnection;
    type Error = DsqlError;

    async fn connect(&self) -> Result<PgConnection> {
        let token = token::generate_token_with_config(
            &self.config.host,
            &self.region,
            &self.config.user,
            &self.sdk_config,
            self.config.token_duration_secs,
        )
        .await?;

        let opts = self.config.to_pg_connect_options(&token);

        PgConnection::connect_with(&opts)
            .await
            .map_err(DsqlError::ConnectionError)
    }

    async fn is_valid(&self, conn: &mut PgConnection) -> Result<()> {
        conn.ping().await.map_err(DsqlError::ConnectionError)
    }

    fn has_broken(&self, _conn: &mut PgConnection) -> bool {
        false
    }
}

pub struct DsqlPool {
    pool: bb8::Pool<DsqlConnectionManager>,
}

impl DsqlPool {
    pub async fn new(conn_str: &str) -> Result<Self> {
        let config = DsqlConfig::from_connection_string(conn_str).await?;
        Self::from_config(config).await
    }

    pub async fn from_config(config: DsqlConfig) -> Result<Self> {
        let sdk_config = config.load_aws_config().await.clone();
        let region = config.resolve_region_with_sdk_config(&sdk_config)?;

        let manager = DsqlConnectionManager {
            config: config.clone(),
            region,
            sdk_config,
        };

        let pool = bb8::Pool::builder()
            .max_size(config.max_connections)
            .max_lifetime(Some(Duration::from_secs(config.max_lifetime_secs)))
            .idle_timeout(Some(Duration::from_secs(config.idle_timeout_secs)))
            .build(manager)
            .await?;

        Ok(Self { pool })
    }

    pub async fn get(&self) -> Result<bb8::PooledConnection<'_, DsqlConnectionManager>> {
        self.pool.get().await.map_err(|e| match e {
            bb8::RunError::User(dsql_err) => dsql_err,
            bb8::RunError::TimedOut => DsqlError::PoolError("connection pool timed out".into()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_new_invalid_url() {
        let result = DsqlPool::new("not-a-url").await;
        assert!(result.is_err(), "Should fail with invalid URL");
    }

    #[tokio::test]
    async fn test_pool_new_invalid_scheme() {
        let result = DsqlPool::new("mysql://admin@example.dsql.us-east-1.on.aws/postgres").await;
        assert!(result.is_err(), "Should fail with non-postgres scheme");
    }

    #[tokio::test]
    async fn test_pool_from_config_creates_pool() {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres",
        )
        .await
        .unwrap();

        // bb8 does not eagerly connect, so pool creation should succeed
        // even without a real database or AWS credentials
        let result = DsqlPool::from_config(config).await;
        assert!(
            result.is_ok(),
            "Pool creation should succeed without eager connections: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_pool_from_config_with_custom_params() {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres?\
             maxConnections=20&maxLifetimeSecs=1800&idleTimeoutSecs=300",
        )
        .await
        .unwrap();

        assert_eq!(config.max_connections, 20);
        assert_eq!(config.max_lifetime_secs, 1800);
        assert_eq!(config.idle_timeout_secs, 300);

        let result = DsqlPool::from_config(config).await;
        assert!(
            result.is_ok(),
            "Pool creation with custom params should succeed: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_pool_get_fails_without_database() {
        let config = DsqlConfig::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres",
        )
        .await
        .unwrap();

        // Pool creation succeeds (no eager connection)
        let pool = match DsqlPool::from_config(config).await {
            Ok(p) => p,
            Err(_) => return, // Skip if pool creation fails
        };

        // Getting a connection should fail — no real database
        let result = pool.get().await;
        assert!(result.is_err(), "get() should fail without a real database");
    }
}
