// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use aurora_dsql_sqlx_connector::{
    DsqlConnectOptions, DsqlError, OCCRetryConfigBuilder, OCCRetryExt, Result,
};
use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
use std::time::Duration;

use super::test_util::build_conn_str;

#[tokio::test]
async fn test_pool_occ_retry_on_concurrent_conflict() -> Result<()> {
    let conn_str = build_conn_str();
    let table_name = format!("occ_test_{}", std::process::id());

    let opts = DsqlConnectOptions::from_connection_string(&conn_str)?;
    let pool = aurora_dsql_sqlx_connector::pool::connect_with(&opts, PgPoolOptions::new()).await?;

    // Setup: create table with a counter
    pool.transaction_with_retry(None, |tx| {
        let t = table_name.clone();
        Box::pin(async move {
            sqlx::query(&format!(
                "CREATE TABLE IF NOT EXISTS {} (id INT PRIMARY KEY, counter INT NOT NULL)",
                t
            ))
            .execute(&mut **tx)
            .await?;
            Ok(())
        })
    })
    .await?;

    // Initialize counter to 0
    pool.transaction_with_retry(None, |tx| {
        let t = table_name.clone();
        Box::pin(async move {
            sqlx::query(&format!("INSERT INTO {} (id, counter) VALUES (1, 0)", t))
                .execute(&mut **tx)
                .await?;
            Ok(())
        })
    })
    .await?;

    // Use higher retry config for high-contention scenario
    let retry_config = OCCRetryConfigBuilder::default()
        .max_attempts(10u32)
        .base_delay_ms(50u64)
        .build()?;

    // Spawn 10 concurrent tasks that all read-modify-write the same counter
    // This creates OCC conflicts that require retry
    let mut handles = Vec::new();
    for _ in 0..10 {
        let pool = pool.clone();
        let table = table_name.clone();
        let config = retry_config.clone();
        handles.push(tokio::spawn(async move {
            pool.transaction_with_retry(Some(&config), |tx| {
                let t = table.clone();
                Box::pin(async move {
                    // Read current value
                    let row = sqlx::query(&format!("SELECT counter FROM {} WHERE id = 1", t))
                        .fetch_one(&mut **tx)
                        .await?;
                    let current: i32 = row.get("counter");

                    // Increment (classic read-modify-write that triggers OCC)
                    sqlx::query(&format!("UPDATE {} SET counter = $1 WHERE id = 1", t))
                        .bind(current + 1)
                        .execute(&mut **tx)
                        .await?;
                    Ok(())
                })
            })
            .await
        }));
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.expect("Task panicked")?;
    }

    // Verify all increments succeeded - counter should be 10
    let row = sqlx::query(&format!("SELECT counter FROM {} WHERE id = 1", table_name))
        .fetch_one(&pool)
        .await
        .map_err(DsqlError::DatabaseError)?;
    let final_count: i32 = row.get("counter");
    assert_eq!(
        final_count, 10,
        "Expected counter to be 10 after concurrent increments with OCC retry"
    );

    // Cleanup
    pool.transaction_with_retry(None, |tx| {
        let t = table_name.clone();
        Box::pin(async move {
            sqlx::query(&format!("DROP TABLE IF EXISTS {}", t))
                .execute(&mut **tx)
                .await?;
            Ok(())
        })
    })
    .await?;

    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn test_pool_background_token_refresh() -> Result<()> {
    // Use a short token duration so the background refresh fires quickly.
    // tokenDurationSecs=6 → refresh_interval = 6 * 4/5 = 4 seconds.
    let conn_str = format!("{}?tokenDurationSecs=6", build_conn_str());

    let opts = DsqlConnectOptions::from_connection_string(&conn_str)?;
    let pool = aurora_dsql_sqlx_connector::pool::connect_with(&opts, PgPoolOptions::new()).await?;

    // Verify the pool works before refresh
    let row = sqlx::query("SELECT 1 as value")
        .fetch_one(&pool)
        .await
        .map_err(DsqlError::DatabaseError)?;
    let value: i32 = row.get("value");
    assert_eq!(value, 1);

    // Wait for the background refresh to fire
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Hold the existing connection so the pool must open a new one
    // using the refreshed token.
    let held_conn = pool.acquire().await.map_err(DsqlError::ConnectionError)?;
    let row = sqlx::query("SELECT 2 as value")
        .fetch_one(&pool)
        .await
        .map_err(DsqlError::DatabaseError)?;
    let value: i32 = row.get("value");
    assert_eq!(
        value, 2,
        "Pool should establish a new connection after token refresh"
    );
    drop(held_conn);

    pool.close().await;
    Ok(())
}
