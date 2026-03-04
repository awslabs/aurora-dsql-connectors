// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::{DsqlError, Result};
use std::time::Duration;

#[cfg(feature = "pool")]
use sqlx::Connection;

#[derive(Debug, Clone)]
pub struct OCCRetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub jitter_factor: f64,
}

impl Default for OCCRetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 100,
            max_delay_ms: 5000,
            jitter_factor: 0.25,
        }
    }
}

/// Detect OCC errors from a raw sqlx::Error (SQLSTATE 40001, OC000, OC001).
pub fn is_occ_error(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        if let Some(code) = db_err.code() {
            let code_str = code.as_ref();
            if code_str == "40001" || code_str == "OC000" || code_str == "OC001" {
                return true;
            }
        }
    }
    false
}

/// Detect OCC errors from a DsqlError (checks wrapped message).
pub fn is_occ_dsql_error(err: &DsqlError) -> bool {
    match err {
        DsqlError::DatabaseError(msg) => {
            msg.contains("OC000") || msg.contains("OC001") || msg.contains("40001")
        }
        _ => false,
    }
}

pub fn calculate_backoff(config: &OCCRetryConfig, attempt: u32) -> Duration {
    let base = config.base_delay_ms as f64;
    let delay = (base * 2_f64.powi(attempt as i32)).min(config.max_delay_ms as f64);
    let jitter = delay * rand::random::<f64>() * config.jitter_factor;

    Duration::from_millis((delay + jitter) as u64)
}

/// Retry an async operation on OCC errors with exponential backoff.
///
/// Re-executes the entire closure on OCC conflict. The closure should
/// contain the full transaction (including `BEGIN`/`COMMIT`) since OCC
/// errors are detected at commit time.
///
/// Returns `DsqlError::OCCRetryExhausted` with the last OCC error
/// preserved as the cause when max attempts are exceeded.
pub async fn retry_on_occ<F, Fut, T>(config: &OCCRetryConfig, f: F) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    if config.max_attempts == 0 {
        return Err(DsqlError::ConfigError(
            "max_attempts must be a positive integer".into(),
        ));
    }

    let mut last_err: Option<DsqlError> = None;
    for attempt in 0..config.max_attempts {
        match f().await {
            Ok(val) => return Ok(val),
            Err(e) if is_occ_dsql_error(&e) => {
                last_err = Some(e);
                if attempt + 1 < config.max_attempts {
                    let delay = calculate_backoff(config, attempt + 1);
                    tokio::time::sleep(delay).await;
                }
            }
            Err(e) => return Err(e),
        }
    }

    let cause = last_err.unwrap();
    Err(DsqlError::OCCRetryExhausted {
        attempts: config.max_attempts,
        message: cause.to_string(),
        source: Box::new(cause),
    })
}

/// Execute a transactional block with automatic OCC retry.
///
/// Gets a connection from the pool, wraps the closure in a transaction,
/// and retries the entire flow on OCC conflict. This is the explicit retry
/// API for custom retry configuration — it bypasses the pool's own
/// `occ_max_retries` to prevent double-retry.
///
/// The closure receives a mutable reference to the transaction and should
/// contain only database operations that are safe to re-execute.
#[cfg(feature = "pool")]
pub async fn with_retry<F, Fut, T>(
    pool: &crate::DsqlPool,
    config: Option<&OCCRetryConfig>,
    f: F,
) -> Result<T>
where
    F: Fn(&mut sqlx::PgConnection) -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let default_config = OCCRetryConfig::default();
    let config = config.unwrap_or(&default_config);

    retry_on_occ(config, || async {
        let mut conn = pool.get().await?;
        let mut tx = conn
            .begin()
            .await
            .map_err(|e| DsqlError::DatabaseError(e.to_string()))?;

        let result = f(&mut *tx).await?;

        tx.commit()
            .await
            .map_err(|e| DsqlError::DatabaseError(e.to_string()))?;

        Ok(result)
    })
    .await
}
