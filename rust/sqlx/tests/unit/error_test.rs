// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use aurora_dsql_sqlx_connector::DsqlError;

#[test]
fn test_error_display() {
    let err = DsqlError::Error("test error".to_string());
    assert_eq!(format!("{}", err), "test error");
}

#[test]
fn test_occ_retry_exhausted_display() {
    let err = DsqlError::OCCRetryExhausted {
        attempts: 3,
        message: "database error: OC000".to_string(),
        source: Box::new(DsqlError::DatabaseError("OC000".to_string())),
    };
    let display = format!("{}", err);
    assert!(display.contains("3 attempts"));
    assert!(display.contains("OC000"));
}
