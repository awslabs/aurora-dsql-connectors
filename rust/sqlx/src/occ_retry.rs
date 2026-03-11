// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::{DsqlError, Result};
use derive_builder::Builder;
use std::time::Duration;

#[cfg(feature = "pool")]
use sqlx::Connection;

#[derive(Debug, Clone, Builder)]
pub struct OCCRetryConfig {
    #[builder(default = "3")]
    pub max_attempts: u32,
    #[builder(default = "100")]
    pub base_delay_ms: u64,
    #[builder(default = "5000")]
    pub max_delay_ms: u64,
    #[builder(default = "0.25")]
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

/// Detect OCC errors by inspecting the SQLSTATE code (40001, OC000, OC001).
pub fn is_occ_error(err: &DsqlError) -> bool {
    match err {
        DsqlError::DatabaseError(sqlx_err) => {
            if let sqlx::Error::Database(db_err) = sqlx_err {
                if let Some(code) = db_err.code() {
                    let c = code.as_ref();
                    return c == "40001" || c == "OC000" || c == "OC001";
                }
            }
            false
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
            Err(e) if is_occ_error(&e) => {
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
        source: Box::new(cause),
    })
}

/// Execute a transactional block with automatic OCC retry.
///
/// Gets a connection from the pool, wraps the closure in a transaction,
/// and retries the entire flow on OCC conflict.
///
/// The closure receives a mutable reference to the transaction and should
/// contain only database operations that are safe to re-execute.
#[cfg(feature = "pool")]
pub async fn with_retry<F, T>(
    pool: &crate::DsqlPool,
    config: Option<&OCCRetryConfig>,
    f: F,
) -> Result<T>
where
    F: for<'c> Fn(
        &'c mut sqlx::PgConnection,
    )
        -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send + 'c>>,
    T: Send,
{
    let default_config = OCCRetryConfig::default();
    let config = config.unwrap_or(&default_config);

    retry_on_occ(config, || async {
        let mut conn = pool.get().await?;
        let mut tx = conn.begin().await.map_err(DsqlError::DatabaseError)?;

        let result = f(&mut tx).await?;

        tx.commit().await.map_err(DsqlError::DatabaseError)?;

        Ok(result)
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;

    #[test]
    fn test_occ_error_detection_sqlstate() {
        let err = DsqlError::DatabaseError(sqlx::Error::Database(Box::new(MockDbError {
            code: Some("40001".to_string()),
            message: "serialization failure".to_string(),
        })));

        assert!(is_occ_error(&err));
    }

    #[test]
    fn test_occ_error_detection_oc000() {
        let err = DsqlError::DatabaseError(sqlx::Error::Database(Box::new(MockDbError {
            code: Some("OC000".to_string()),
            message: "optimistic concurrency failure".to_string(),
        })));

        assert!(is_occ_error(&err));
    }

    #[test]
    fn test_occ_error_detection_oc001() {
        let err = DsqlError::DatabaseError(sqlx::Error::Database(Box::new(MockDbError {
            code: Some("OC001".to_string()),
            message: "transaction conflict".to_string(),
        })));

        assert!(is_occ_error(&err));
    }

    #[test]
    fn test_non_occ_error() {
        let err = DsqlError::DatabaseError(sqlx::Error::Database(Box::new(MockDbError {
            code: Some("23505".to_string()),
            message: "unique violation".to_string(),
        })));

        assert!(!is_occ_error(&err));
    }

    #[test]
    fn test_backoff_calculation() {
        let config = OCCRetryConfig::default();

        let delay1 = calculate_backoff(&config, 1);
        assert!(delay1 >= Duration::from_millis(200));
        assert!(delay1 <= Duration::from_millis(250));

        let delay2 = calculate_backoff(&config, 2);
        assert!(delay2 >= Duration::from_millis(400));
        assert!(delay2 <= Duration::from_millis(500));
    }

    #[test]
    fn test_backoff_max_delay() {
        let config = OCCRetryConfig::default();

        let delay = calculate_backoff(&config, 10);
        assert!(delay <= Duration::from_millis(6250)); // max_delay + 25% jitter
    }

    #[test]
    fn test_builder_defaults() {
        let config = OCCRetryConfigBuilder::default().build().unwrap();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.base_delay_ms, 100);
        assert_eq!(config.max_delay_ms, 5000);
        assert!((config.jitter_factor - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn test_builder_custom_values() {
        let config = OCCRetryConfigBuilder::default()
            .max_attempts(5u32)
            .base_delay_ms(200u64)
            .build()
            .unwrap();
        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.base_delay_ms, 200);
        assert_eq!(config.max_delay_ms, 5000); // default
    }

    #[test]
    fn test_is_occ_error_non_database() {
        let sqlx_err = sqlx::Error::Protocol("connection refused".into());
        let err = DsqlError::ConnectionError(sqlx_err);
        assert!(!is_occ_error(&err));
    }

    #[test]
    fn test_is_occ_error_no_sqlstate_code() {
        let err = DsqlError::DatabaseError(sqlx::Error::Database(Box::new(MockDbError {
            code: None,
            message: "unknown error".to_string(),
        })));
        assert!(!is_occ_error(&err));
    }

    #[test]
    fn test_occ_retry_exhausted_preserves_cause() {
        let sqlx_err = sqlx::Error::Database(Box::new(MockDbError {
            code: Some("OC000".to_string()),
            message: "OC000 conflict".to_string(),
        }));
        let err = DsqlError::OCCRetryExhausted {
            attempts: 3,
            source: Box::new(DsqlError::DatabaseError(sqlx_err)),
        };
        assert!(err.to_string().contains("3 attempts"));
        assert!(err.to_string().contains("OC000"));
        // Verify source is accessible via std::error::Error
        let std_err: &dyn std::error::Error = &err;
        assert!(std_err.source().is_some());
    }

    fn make_occ_error() -> DsqlError {
        DsqlError::DatabaseError(sqlx::Error::Database(Box::new(MockDbError {
            code: Some("OC000".to_string()),
            message: "mutation conflict".to_string(),
        })))
    }

    fn make_non_occ_error() -> DsqlError {
        DsqlError::DatabaseError(sqlx::Error::Database(Box::new(MockDbError {
            code: Some("23505".to_string()),
            message: "unique violation".to_string(),
        })))
    }

    #[tokio::test]
    async fn test_retry_on_occ_succeeds_first_try() {
        let config = OCCRetryConfig::default();
        let calls = AtomicU32::new(0);

        let result = retry_on_occ(&config, || async {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok::<&str, DsqlError>("done")
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "done");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_on_occ_retries_then_succeeds() {
        let config = OCCRetryConfigBuilder::default()
            .max_attempts(3u32)
            .base_delay_ms(1u64)
            .build()
            .unwrap();
        let calls = AtomicU32::new(0);

        let result = retry_on_occ(&config, || async {
            let attempt = calls.fetch_add(1, Ordering::SeqCst);
            if attempt < 2 {
                Err(make_occ_error())
            } else {
                Ok("recovered")
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "recovered");
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_on_occ_exhausted() {
        let config = OCCRetryConfigBuilder::default()
            .max_attempts(2u32)
            .base_delay_ms(1u64)
            .build()
            .unwrap();
        let calls = AtomicU32::new(0);

        let result: Result<()> = retry_on_occ(&config, || async {
            calls.fetch_add(1, Ordering::SeqCst);
            Err(make_occ_error())
        })
        .await;

        assert!(result.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        match result.unwrap_err() {
            DsqlError::OCCRetryExhausted { attempts, .. } => assert_eq!(attempts, 2),
            other => panic!("Expected OCCRetryExhausted, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_retry_on_occ_non_occ_error_no_retry() {
        let config = OCCRetryConfig::default();
        let calls = AtomicU32::new(0);

        let result: Result<()> = retry_on_occ(&config, || async {
            calls.fetch_add(1, Ordering::SeqCst);
            Err(make_non_occ_error())
        })
        .await;

        assert!(result.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_on_occ_zero_attempts() {
        let config = OCCRetryConfigBuilder::default()
            .max_attempts(0u32)
            .build()
            .unwrap();

        let result: Result<()> = retry_on_occ(&config, || async { Ok(()) }).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            DsqlError::ConfigError(msg) => assert!(msg.contains("positive")),
            other => panic!("Expected ConfigError, got: {:?}", other),
        }
    }

    struct MockDbError {
        code: Option<String>,
        message: String,
    }

    impl std::fmt::Display for MockDbError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl std::fmt::Debug for MockDbError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl std::error::Error for MockDbError {}

    impl sqlx::error::DatabaseError for MockDbError {
        fn message(&self) -> &str {
            &self.message
        }

        fn code(&self) -> Option<std::borrow::Cow<'_, str>> {
            self.code
                .as_ref()
                .map(|s| std::borrow::Cow::Borrowed(s.as_str()))
        }

        fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
            self
        }

        fn as_error_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
            self
        }

        fn into_error(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync + 'static> {
            self
        }

        fn kind(&self) -> sqlx::error::ErrorKind {
            sqlx::error::ErrorKind::Other
        }
    }
}
