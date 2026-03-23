// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use aurora_dsql_sqlx_connector::{DsqlConnectOptions, DsqlError, Result};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Connection, Row};
use std::sync::Arc;
use std::time::Duration;

use super::test_util::build_conn_str;

#[tokio::test]
async fn test_pool_transactional_write() -> Result<()> {
    let conn_str = build_conn_str();
    let table_name = format!("tx_test_{}", std::process::id());

    let opts = DsqlConnectOptions::from_connection_string(&conn_str)?;
    let pool = aurora_dsql_sqlx_connector::pool::connect_with(&opts, PgPoolOptions::new()).await?;

    // Setup
    sqlx::query(&format!(
        "CREATE TABLE IF NOT EXISTS {} (id INT PRIMARY KEY, name TEXT)",
        table_name
    ))
    .execute(&pool)
    .await
    .unwrap();

    // Manual transaction
    {
        let mut conn = pool.acquire().await.map_err(DsqlError::ConnectionError)?;
        let mut tx = conn.begin().await.map_err(DsqlError::DatabaseError)?;

        sqlx::query(&format!(
            "INSERT INTO {} (id, name) VALUES (1, 'alice')",
            table_name
        ))
        .execute(&mut *tx)
        .await
        .unwrap();

        tx.commit().await.map_err(DsqlError::DatabaseError)?;
    }

    // Verify the write persisted
    let row = sqlx::query(&format!("SELECT name FROM {} WHERE id = 1", table_name))
        .fetch_one(&pool)
        .await
        .unwrap();
    let name: String = row.get("name");
    assert_eq!(name, "alice");

    // Cleanup
    sqlx::query(&format!("DROP TABLE {}", table_name))
        .execute(&pool)
        .await
        .unwrap();

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

    // Verify the pool still works after token refresh
    let row = sqlx::query("SELECT 2 as value")
        .fetch_one(&pool)
        .await
        .map_err(DsqlError::DatabaseError)?;
    let value: i32 = row.get("value");
    assert_eq!(value, 2, "Pool should work after background token refresh");

    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn test_pool_occ_concurrent_conflict() -> Result<()> {
    let table_name = format!("occ_test_{}", std::process::id());

    let opts = DsqlConnectOptions::from_connection_string(&build_conn_str())?;
    let pool = Arc::new(
        aurora_dsql_sqlx_connector::pool::connect_with(&opts, PgPoolOptions::new()).await?,
    );

    let occ_config = aurora_dsql_sqlx_connector::OCCRetryConfigBuilder::default()
        .max_attempts(5u32)
        .build()
        .unwrap();

    // Setup: create table and seed a row
    sqlx::query(&format!(
        "CREATE TABLE IF NOT EXISTS {} (id INT PRIMARY KEY, counter INT NOT NULL)",
        table_name
    ))
    .execute(&*pool)
    .await
    .map_err(DsqlError::DatabaseError)?;

    sqlx::query(&format!(
        "INSERT INTO {} (id, counter) VALUES (1, 0)",
        table_name
    ))
    .execute(&*pool)
    .await
    .map_err(DsqlError::DatabaseError)?;

    // Spawn two tasks that both UPDATE the same row concurrently
    // with OCC retry via retry_on_occ.
    let table = table_name.clone();
    let pool1 = Arc::clone(&pool);
    let occ1 = occ_config.clone();
    let handle1 = tokio::spawn(async move {
        aurora_dsql_sqlx_connector::retry_on_occ(&occ1, || {
            let p = pool1.clone();
            let t = table.clone();
            async move {
                let mut conn = p.acquire().await?;
                let mut tx = conn.begin().await?;
                sqlx::query(&format!(
                    "UPDATE {} SET counter = counter + 1 WHERE id = 1",
                    t
                ))
                .execute(&mut *tx)
                .await?;
                tx.commit().await?;
                Ok(())
            }
        })
        .await
    });

    let table = table_name.clone();
    let pool2 = Arc::clone(&pool);
    let occ2 = occ_config.clone();
    let handle2 = tokio::spawn(async move {
        aurora_dsql_sqlx_connector::retry_on_occ(&occ2, || {
            let p = pool2.clone();
            let t = table.clone();
            async move {
                let mut conn = p.acquire().await?;
                let mut tx = conn.begin().await?;
                sqlx::query(&format!(
                    "UPDATE {} SET counter = counter + 1 WHERE id = 1",
                    t
                ))
                .execute(&mut *tx)
                .await?;
                tx.commit().await?;
                Ok(())
            }
        })
        .await
    });

    handle1.await.expect("task 1 panicked")?;
    handle2.await.expect("task 2 panicked")?;

    // Both increments should have succeeded
    let row = sqlx::query(&format!("SELECT counter FROM {} WHERE id = 1", table_name))
        .fetch_one(&*pool)
        .await
        .unwrap();
    let counter: i32 = row.get("counter");
    assert_eq!(counter, 2, "Both concurrent updates should have committed");

    // Cleanup
    sqlx::query(&format!("DROP TABLE {}", table_name))
        .execute(&*pool)
        .await
        .map_err(DsqlError::DatabaseError)?;

    pool.close().await;
    Ok(())
}
