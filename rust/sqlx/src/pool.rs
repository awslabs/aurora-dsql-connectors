// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use crate::{
    occ_retry::OCCRetryConfig, token, util::Region, DsqlConfig, DsqlError, DsqlPoolConfig, Result,
};
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
    occ_retry_config: Option<OCCRetryConfig>,
}

impl DsqlPool {
    pub async fn new(conn_str: &str) -> Result<Self> {
        let config = DsqlPoolConfig::from_connection_string(conn_str)?;
        Self::from_config(config).await
    }

    pub async fn from_config(config: DsqlPoolConfig) -> Result<Self> {
        let mut connection = config.connection;
        let sdk_config = connection.load_aws_config().await;
        connection.host = connection.resolve_host(&sdk_config)?;
        let region = connection.resolve_region(&sdk_config)?;

        let occ_retry_config = config.occ_max_retries.map(|max_retries| OCCRetryConfig {
            max_attempts: max_retries,
            ..OCCRetryConfig::default()
        });

        let manager = DsqlConnectionManager {
            config: connection,
            region,
            sdk_config,
        };

        let pool = bb8::Pool::builder()
            .max_size(config.max_connections)
            .max_lifetime(Some(Duration::from_secs(config.max_lifetime_secs)))
            .idle_timeout(Some(Duration::from_secs(config.idle_timeout_secs)))
            .build(manager)
            .await?;

        Ok(Self {
            pool,
            occ_retry_config,
        })
    }

    /// Get a raw connection from the pool.
    ///
    /// This bypasses OCC retry — callers are responsible for handling
    /// OCC errors (SQLSTATE 40001/OC000/OC001) themselves. Prefer
    /// [`with()`](Self::with) for write operations that need automatic retry.
    pub async fn get(&self) -> Result<bb8::PooledConnection<'_, DsqlConnectionManager>> {
        self.pool.get().await.map_err(|e| match e {
            bb8::RunError::User(dsql_err) => dsql_err,
            bb8::RunError::TimedOut => DsqlError::PoolError("connection pool timed out".into()),
        })
    }

    /// Execute a transactional block with automatic OCC retry.
    ///
    /// If `occ_max_retries` was set in the pool config, the closure is
    /// retried on OCC conflict with exponential backoff. Otherwise
    /// the closure runs once without retry.
    ///
    /// The closure receives a mutable reference to the active transaction
    /// and should contain only operations that are safe to re-execute.
    pub async fn with<F, T>(&self, f: F) -> Result<T>
    where
        F: for<'c> Fn(
            &'c mut sqlx::PgConnection,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<T>> + Send + 'c>,
        >,
        T: Send,
    {
        let run = || async {
            let mut conn = self.get().await?;
            let mut tx = conn.begin().await.map_err(DsqlError::DatabaseError)?;
            let result = f(&mut tx).await?;
            tx.commit().await.map_err(DsqlError::DatabaseError)?;
            Ok(result)
        };

        match &self.occ_retry_config {
            Some(config) => crate::occ_retry::retry_on_occ(config, run).await,
            None => run().await,
        }
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
        let config = DsqlPoolConfig::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres",
        )
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
        let config = DsqlPoolConfig::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres?\
             maxConnections=20&maxLifetimeSecs=1800&idleTimeoutSecs=300",
        )
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
    async fn test_pool_from_config_with_occ_max_retries() {
        let config = DsqlPoolConfig::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres?occMaxRetries=5",
        )
        .unwrap();

        assert_eq!(config.occ_max_retries, Some(5));

        let pool = DsqlPool::from_config(config).await.unwrap();
        assert!(pool.occ_retry_config.is_some());
        assert_eq!(pool.occ_retry_config.as_ref().unwrap().max_attempts, 5);
    }

    #[tokio::test]
    async fn test_pool_from_config_without_occ_max_retries() {
        let config = DsqlPoolConfig::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres",
        )
        .unwrap();

        let pool = DsqlPool::from_config(config).await.unwrap();
        assert!(pool.occ_retry_config.is_none());
    }

    #[tokio::test]
    async fn test_pool_get_fails_without_database() {
        let config = DsqlPoolConfig::from_connection_string(
            "postgres://admin@example.dsql.us-east-1.on.aws/postgres",
        )
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
