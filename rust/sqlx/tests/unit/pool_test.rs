// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use aurora_dsql_sqlx_connector::{DsqlConfig, DsqlPool};

#[tokio::test]
async fn test_pool_new_invalid_url() {
    let result = DsqlPool::new("not-a-url").await;
    assert!(result.is_err(), "Should fail with invalid URL");
}

#[tokio::test]
async fn test_pool_new_invalid_scheme() {
    let result =
        DsqlPool::new("mysql://admin@example.dsql.us-east-1.on.aws/postgres").await;
    assert!(result.is_err(), "Should fail with non-postgres scheme");
}

#[tokio::test]
async fn test_pool_from_config_creates_pool() {
    let config = DsqlConfig::from_connection_string(
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
    let config = DsqlConfig::from_connection_string(
        "postgres://admin@example.dsql.us-east-1.on.aws/postgres?\
         maxConnections=20&maxLifetimeSecs=1800&idleTimeoutSecs=300&occMaxRetries=5",
    )
    .unwrap();

    assert_eq!(config.max_connections, 20);
    assert_eq!(config.max_lifetime_secs, 1800);
    assert_eq!(config.idle_timeout_secs, 300);
    assert_eq!(config.occ_max_retries, Some(5));

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