// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use aurora_dsql_sqlx_connector::{DsqlConnectOptions, DsqlError, OCCRetryConfigBuilder, Result};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Acquire, Row};
use std::time::Duration;

use super::test_util::build_conn_str;

#[tokio::test]
async fn test_pool_transactional_write() -> Result<()> {
    let conn_str = build_conn_str();
    let table_name = format!("tx_test_{}", std::process::id());

    let opts = DsqlConnectOptions::from_connection_string(&conn_str)?;
    let pool = aurora_dsql_sqlx_connector::pool::connect_with(&opts, PgPoolOptions::new()).await?;

    let occ_config = OCCRetryConfigBuilder::default()
        .max_attempts(10)
        .base_delay_ms(100)
        .build()
        .unwrap();

    // Setup: create table with OCC retry
    aurora_dsql_sqlx_connector::retry_on_occ(&occ_config, || {
        let p = pool.clone();
        let t = table_name.clone();
        async move {
            let mut conn = p.acquire().await?;
            let mut tx = conn.begin().await?;
            sqlx::query(&format!(
                "CREATE TABLE IF NOT EXISTS {} (id INT PRIMARY KEY, name TEXT)",
                t
            ))
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            Ok(())
        }
    })
    .await?;

    // Transactional write with OCC retry
    aurora_dsql_sqlx_connector::retry_on_occ(&occ_config, || {
        let p = pool.clone();
        let t = table_name.clone();
        async move {
            let mut conn = p.acquire().await?;
            let mut tx = conn.begin().await?;
            sqlx::query(&format!("INSERT INTO {} (id, name) VALUES (1, 'alice')", t))
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            Ok(())
        }
    })
    .await?;

    // Verify the write persisted
    let row = sqlx::query(&format!("SELECT name FROM {} WHERE id = 1", table_name))
        .fetch_one(&pool)
        .await
        .map_err(DsqlError::DatabaseError)?;
    let name: String = row.get("name");
    assert_eq!(name, "alice");

    // Cleanup
    aurora_dsql_sqlx_connector::retry_on_occ(&occ_config, || {
        let p = pool.clone();
        let t = table_name.clone();
        async move {
            let mut conn = p.acquire().await?;
            let mut tx = conn.begin().await?;
            sqlx::query(&format!("DROP TABLE IF EXISTS {}", t))
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            Ok(())
        }
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
