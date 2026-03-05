// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use crate::occ_retry::{self, OCCRetryConfig};
use crate::token;
use crate::{DsqlConfig, DsqlError, Result};
use sqlx::{Connection, Executor, PgConnection};

// -- Connection Manager --

pub struct DsqlConnectionManager {
    config: DsqlConfig,
    region: String,
    sdk_config: aws_config::SdkConfig,
}

impl bb8::ManageConnection for DsqlConnectionManager {
    type Connection = PgConnection;
    type Error = DsqlError;

    async fn connect(&self) -> std::result::Result<PgConnection, DsqlError> {
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
            .map_err(|e| DsqlError::ConnectionError(e.to_string()))
    }

    async fn is_valid(&self, conn: &mut PgConnection) -> std::result::Result<(), DsqlError> {
        conn.ping()
            .await
            .map_err(|e| DsqlError::ConnectionError(e.to_string()))
    }

    fn has_broken(&self, _conn: &mut PgConnection) -> bool {
        false
    }
}

// -- Pool --

pub struct DsqlPool {
    pool: bb8::Pool<DsqlConnectionManager>,
    occ_max_retries: Option<u32>,
    retry_config: OCCRetryConfig,
}

impl DsqlPool {
    pub async fn new(conn_str: &str) -> Result<Self> {
        let config = DsqlConfig::from_connection_string(conn_str)?;
        Self::from_config(config).await
    }

    pub async fn from_config(config: DsqlConfig) -> Result<Self> {
        let region = config.resolve_region().await?;
        let sdk_config = config.load_aws_config().await;
        let occ_max_retries = config.occ_max_retries;

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
            .await
            .map_err(|e| DsqlError::PoolError(e.to_string()))?;

        Ok(Self {
            pool,
            occ_max_retries,
            retry_config: OCCRetryConfig::default(),
        })
    }

    pub async fn get(
        &self,
    ) -> std::result::Result<
        bb8::PooledConnection<'_, DsqlConnectionManager>,
        DsqlError,
    > {
        self.pool
            .get()
            .await
            .map_err(|e| DsqlError::PoolError(e.to_string()))
    }

    pub async fn execute(&self, sql: &str) -> Result<u64> {
        self.retry_on_occ(|| async {
            let mut conn = self.get().await?;
            conn.execute(sql)
                .await
                .map(|r| r.rows_affected())
                .map_err(|e| DsqlError::DatabaseError(e.to_string()))
        })
        .await
    }

    pub async fn fetch_one(
        &self,
        sql: &str,
    ) -> Result<sqlx::postgres::PgRow> {
        self.retry_on_occ(|| async {
            let mut conn = self.get().await?;
            conn.fetch_one(sql)
                .await
                .map_err(|e| DsqlError::DatabaseError(e.to_string()))
        })
        .await
    }

    pub async fn fetch_all(
        &self,
        sql: &str,
    ) -> Result<Vec<sqlx::postgres::PgRow>> {
        self.retry_on_occ(|| async {
            let mut conn = self.get().await?;
            conn.fetch_all(sql)
                .await
                .map_err(|e| DsqlError::DatabaseError(e.to_string()))
        })
        .await
    }

    /// Execute a single SQL statement with an explicit OCC retry count.
    /// Useful for DDL or one-off statements that need retry independently
    /// of the pool's `occ_max_retries` setting.
    pub async fn exec_with_retry(&self, sql: &str, max_retries: u32) -> Result<u64> {
        let config = OCCRetryConfig {
            max_attempts: max_retries,
            ..self.retry_config.clone()
        };
        occ_retry::retry_on_occ(&config, || async {
            let mut conn = self.get().await?;
            conn.execute(sql)
                .await
                .map(|r| r.rows_affected())
                .map_err(|e| DsqlError::DatabaseError(e.to_string()))
        })
        .await
    }

    async fn retry_on_occ<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let max_retries = match self.occ_max_retries {
            Some(n) => n,
            None => return f().await,
        };

        let config = OCCRetryConfig {
            max_attempts: max_retries,
            ..self.retry_config.clone()
        };
        occ_retry::retry_on_occ(&config, f).await
    }
}
