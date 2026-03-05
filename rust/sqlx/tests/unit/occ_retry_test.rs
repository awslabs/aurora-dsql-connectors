// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use aurora_dsql_sqlx_connector::occ_retry::{
    calculate_backoff, is_occ_dsql_error, is_occ_error, OCCRetryConfig,
};
use aurora_dsql_sqlx_connector::DsqlError;
use std::time::Duration;

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
fn test_is_occ_dsql_error_oc000() {
    let err = DsqlError::DatabaseError("ERROR: OC000 mutation conflict".into());
    assert!(is_occ_dsql_error(&err));
}

#[test]
fn test_is_occ_dsql_error_non_occ() {
    let err = DsqlError::DatabaseError("unique violation".into());
    assert!(!is_occ_dsql_error(&err));
}

#[test]
fn test_is_occ_dsql_error_non_database() {
    let err = DsqlError::ConnectionError("connection refused".into());
    assert!(!is_occ_dsql_error(&err));
}

#[test]
fn test_occ_retry_exhausted_preserves_cause() {
    let cause = DsqlError::DatabaseError("OC000 conflict".into());
    let err = DsqlError::OCCRetryExhausted {
        attempts: 3,
        message: cause.to_string(),
        source: Box::new(DsqlError::DatabaseError("OC000 conflict".into())),
    };
    assert!(err.to_string().contains("3 attempts"));
    assert!(err.to_string().contains("OC000"));
    // Verify source is accessible via std::error::Error
    let std_err: &dyn std::error::Error = &err;
    assert!(std_err.source().is_some());
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
