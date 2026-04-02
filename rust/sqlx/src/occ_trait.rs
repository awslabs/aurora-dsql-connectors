// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

//! Trait-based OCC retry with automatic query retry.

use crate::occ_retry::OCCRetryConfig;
use crate::retry_query::RetryQuery;
use crate::{DsqlError, Result};
use sqlx::pool::PoolConnection;
use sqlx::postgres::{PgPool, Postgres};
use sqlx::Transaction;

/// Trait for executing queries with automatic OCC retry.
///
/// Implemented for `PgPool` with default configuration. For custom retry
/// configuration, use `pool::connect_with_retry()` to get a `RetryPool<PgPool>`.
pub trait RetryExecutor: Clone + Send + Sync {
    /// Override to customize retry behavior.
    fn retry_config(&self) -> OCCRetryConfig {
        OCCRetryConfig::default()
    }

    /// Create a query with automatic OCC retry.
    fn query<'q>(&self, sql: &'q str) -> RetryQuery<'q, Self> {
        RetryQuery::new(sql, self.clone(), self.retry_config())
    }
}

/// Built-in implementation for sqlx PgPool with default retry configuration.
impl RetryExecutor for PgPool {}

/// Pool wrapper with automatic OCC retry via `RetryExecutor`.
///
/// Create via `pool::connect_with_retry()` to specify custom retry configuration.
/// Use `.query()` for automatic retry, or `.begin()`/`.acquire()` for transactions.

#[derive(Clone)]
#[non_exhaustive]
pub struct RetryPool<P> {
    inner: P,
    config: OCCRetryConfig,
}

impl<P> RetryPool<P> {
    /// Create a new RetryPool wrapping an executor with a retry configuration.
    pub fn new(pool: P, config: OCCRetryConfig) -> Self {
        Self {
            inner: pool,
            config,
        }
    }

    /// Get a reference to the inner pool.
    pub fn inner(&self) -> &P {
        &self.inner
    }

    /// Get a reference to the retry configuration.
    pub fn config(&self) -> &OCCRetryConfig {
        &self.config
    }
}

impl<P: RetryExecutor> RetryExecutor for RetryPool<P> {
    fn retry_config(&self) -> OCCRetryConfig {
        self.config.clone()
    }
}

/// Convenience methods for `RetryPool<PgPool>` to delegate common operations.
impl RetryPool<PgPool> {
    /// Create a query with automatic OCC retry using this pool's custom configuration.
    ///
    /// This shadows the trait default to ensure the executor passed to `RetryQuery`
    /// is the inner `PgPool` (which satisfies the `Executor` bound), not the wrapper.
    pub fn query<'q>(&self, sql: &'q str) -> RetryQuery<'q, PgPool> {
        RetryQuery::new(sql, self.inner.clone(), self.retry_config())
    }

    /// Begin a transaction. Use `.query()` for automatic retry on single statements.
    ///
    /// Note: Transactions should wrap retry logic at the transaction level using
    /// `retry_on_occ()` rather than retrying individual statements within a transaction.
    pub async fn begin(&self) -> Result<Transaction<'_, Postgres>> {
        self.inner.begin().await.map_err(DsqlError::DatabaseError)
    }

    /// Acquire a connection from the pool.
    pub async fn acquire(&self) -> Result<PoolConnection<Postgres>> {
        self.inner.acquire().await.map_err(DsqlError::DatabaseError)
    }

    /// Close the underlying pool and stop the background token refresh task.
    ///
    /// This will wait for all connections to be returned to the pool and then
    /// close them. The background task that refreshes IAM tokens will also be stopped.
    pub async fn close(&self) {
        self.inner.close().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_pool_stores_config() {
        let pool = PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let config = OCCRetryConfig::default();
        let retry_pool = RetryPool::new(pool, config.clone());

        assert_eq!(retry_pool.config().max_attempts(), config.max_attempts());
    }

    #[tokio::test]
    async fn test_retry_pool_implements_retry_executor() {
        let pool = PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let config = OCCRetryConfig::default();
        let retry_pool = RetryPool::new(pool, config);

        // Should be able to call .query() via trait
        let _query = retry_pool.query("SELECT 1");
    }

    #[tokio::test]
    async fn test_retry_pool_clone() {
        let pool = PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let config = OCCRetryConfig::default();
        let retry_pool = RetryPool::new(pool, config);

        let cloned = retry_pool.clone();
        assert_eq!(
            cloned.config().max_attempts(),
            retry_pool.config().max_attempts()
        );
    }

    #[tokio::test]
    async fn test_pgpool_implements_retry_executor() {
        let pool = PgPool::connect_lazy("postgres://localhost/test").unwrap();

        // Should be able to call .query() via trait
        let _query = pool.query("SELECT 1");
    }
}
