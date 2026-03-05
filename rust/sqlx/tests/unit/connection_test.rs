// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use aurora_dsql_sqlx_connector::{DsqlConnection, DsqlError};

#[tokio::test]
async fn test_connect_with_invalid_url() {
    let result = DsqlConnection::connect_with("not-a-url").await;
    assert!(result.is_err(), "Should fail with invalid URL");
}

#[tokio::test]
async fn test_connect_with_invalid_scheme() {
    let result =
        DsqlConnection::connect_with("mysql://admin@example.dsql.us-east-1.on.aws/postgres").await;
    assert!(result.is_err(), "Should fail with non-postgres scheme");
}

#[tokio::test]
async fn test_connect_with_empty_string() {
    let result = DsqlConnection::connect_with("").await;
    assert!(result.is_err(), "Should fail with empty connection string");
}

#[tokio::test]
async fn test_connect_with_unreachable_host() {
    // Valid connection string but no real database — fails at token generation
    // (no credentials) or connection (unreachable host)
    let result =
        DsqlConnection::connect_with("postgres://admin@example.dsql.us-east-1.on.aws/postgres")
            .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    match &err {
        DsqlError::TokenError(_) | DsqlError::ConnectionError(_) => {}
        other => panic!(
            "Expected TokenError or ConnectionError, got: {:?}",
            other
        ),
    }
}

#[tokio::test]
async fn test_connect_from_config() {
    // Test the connect(config) path
    let config = aurora_dsql_sqlx_connector::DsqlConfig::from_connection_string(
        "postgres://admin@example.dsql.us-east-1.on.aws/postgres",
    )
    .unwrap();

    let result = DsqlConnection::connect(&config).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    match &err {
        DsqlError::TokenError(_) | DsqlError::ConnectionError(_) => {}
        other => panic!(
            "Expected TokenError or ConnectionError, got: {:?}",
            other
        ),
    }
}