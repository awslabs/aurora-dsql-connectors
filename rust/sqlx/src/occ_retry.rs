// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::{DsqlError, Result};
use derive_builder::Builder;
use std::time::Duration;

#[derive(Debug, Clone, Builder)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct OCCRetryConfig {
    #[builder(default = "3")]
    max_attempts: u32,
    #[builder(default = "100")]
    base_delay_ms: u64,
    #[builder(default = "5000")]
    max_delay_ms: u64,
    #[builder(default = "0.25")]
    jitter_factor: f64,
}

impl OCCRetryConfig {
    /// Maximum number of retry attempts.
    pub fn max_attempts(&self) -> u32 {
        self.max_attempts
    }

    /// Base delay in milliseconds for exponential backoff.
    pub fn base_delay_ms(&self) -> u64 {
        self.base_delay_ms
    }

    /// Maximum delay in milliseconds.
    pub fn max_delay_ms(&self) -> u64 {
        self.max_delay_ms
    }

    /// Jitter factor (0.0 to 1.0) applied to backoff delays.
    pub fn jitter_factor(&self) -> f64 {
        self.jitter_factor
    }
}

impl OCCRetryConfigBuilder {
    fn validate(&self) -> std::result::Result<(), String> {
        if let Some(0) = self.max_attempts {
            return Err("max_attempts must be greater than 0".into());
        }
        Ok(())
    }
}

impl Default for OCCRetryConfig {
    fn default() -> Self {
        OCCRetryConfigBuilder::default()
            .build()
            .expect("default builder values are valid")
    }
}

/// Detect OCC errors by inspecting the SQLSTATE code (40001, OC000, OC001).
pub fn is_occ_error(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        matches!(db_err.code().as_deref(), Some("40001" | "OC000" | "OC001"))
    } else {
        false
    }
}

fn calculate_backoff(config: &OCCRetryConfig, attempt: u32) -> Duration {
    let base = config.base_delay_ms as f64;
    let delay = (base * 2_f64.powi((attempt - 1) as i32)).min(config.max_delay_ms as f64);
    let jitter = delay * rand::random::<f64>() * config.jitter_factor;

    Duration::from_millis((delay + jitter) as u64)
}

/// Retry an async operation on OCC errors with exponential backoff.
///
/// The closure returns `Result<T, sqlx::Error>` directly — no need to
/// wrap errors in `DsqlError`. The helper handles error mapping internally.
///
/// Re-executes the entire closure on OCC conflict. The closure should
/// contain the full transaction (including `BEGIN`/`COMMIT`) since OCC
/// errors are detected at commit time.
///
/// **Warning:** The closure must be idempotent — it may be called multiple
/// times if OCC conflicts occur. Avoid side effects (e.g. sending emails,
/// incrementing external counters) that should not be repeated.
///
/// Returns `DsqlError::OCCRetryExhausted` with the last OCC error
/// preserved as the cause when max attempts are exceeded.
pub async fn retry_on_occ<F, Fut, T>(config: &OCCRetryConfig, f: F) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = std::result::Result<T, sqlx::Error>>,
{
    let max_attempts = config.max_attempts;
    let mut attempt = 1;

    loop {
        match f().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                if !is_occ_error(&e) {
                    return Err(DsqlError::DatabaseError(e));
                }

                if attempt == max_attempts {
                    return Err(DsqlError::OCCRetryExhausted {
                        attempts: max_attempts,
                        source: Box::new(DsqlError::DatabaseError(e)),
                    });
                }

                let delay = calculate_backoff(config, attempt);
                tokio::time::sleep(delay).await;

                attempt += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;

    // --- Test helpers ---

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

    fn make_occ_error() -> sqlx::Error {
        sqlx::Error::Database(Box::new(MockDbError {
            code: Some("OC000".to_string()),
            message: "mutation conflict".to_string(),
        }))
    }

    fn make_non_occ_error() -> sqlx::Error {
        sqlx::Error::Database(Box::new(MockDbError {
            code: Some("23505".to_string()),
            message: "unique violation".to_string(),
        }))
    }

    // --- Tests ---

    #[test]
    fn test_occ_error_detection_sqlstate() {
        let err = sqlx::Error::Database(Box::new(MockDbError {
            code: Some("40001".to_string()),
            message: "serialization failure".to_string(),
        }));

        assert!(is_occ_error(&err));
    }

    #[test]
    fn test_occ_error_detection_oc000() {
        let err = sqlx::Error::Database(Box::new(MockDbError {
            code: Some("OC000".to_string()),
            message: "optimistic concurrency failure".to_string(),
        }));

        assert!(is_occ_error(&err));
    }

    #[test]
    fn test_occ_error_detection_oc001() {
        let err = sqlx::Error::Database(Box::new(MockDbError {
            code: Some("OC001".to_string()),
            message: "transaction conflict".to_string(),
        }));

        assert!(is_occ_error(&err));
    }

    #[test]
    fn test_non_occ_error() {
        let err = sqlx::Error::Database(Box::new(MockDbError {
            code: Some("23505".to_string()),
            message: "unique violation".to_string(),
        }));

        assert!(!is_occ_error(&err));
    }

    #[test]
    fn test_backoff_calculation() {
        let config = OCCRetryConfig::default();

        let delay1 = calculate_backoff(&config, 1);
        assert!(delay1 >= Duration::from_millis(100));
        assert!(delay1 <= Duration::from_millis(125));

        let delay2 = calculate_backoff(&config, 2);
        assert!(delay2 >= Duration::from_millis(200));
        assert!(delay2 <= Duration::from_millis(250));
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
        let err = sqlx::Error::Protocol("connection refused".into());
        assert!(!is_occ_error(&err));
    }

    #[test]
    fn test_is_occ_error_no_sqlstate_code() {
        let err = sqlx::Error::Database(Box::new(MockDbError {
            code: None,
            message: "unknown error".to_string(),
        }));
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

    #[tokio::test]
    async fn test_retry_on_occ_succeeds_first_try() {
        let config = OCCRetryConfig::default();
        let calls = AtomicU32::new(0);

        let result = retry_on_occ(&config, || async {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok::<&str, sqlx::Error>("done")
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
            Err::<(), sqlx::Error>(make_occ_error())
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
            Err::<(), sqlx::Error>(make_non_occ_error())
        })
        .await;

        assert!(result.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_builder_rejects_zero_attempts() {
        let result = OCCRetryConfigBuilder::default().max_attempts(0u32).build();

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("max_attempts"),
            "Expected max_attempts error, got: {}",
            err
        );
    }
}
