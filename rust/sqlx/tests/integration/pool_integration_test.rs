// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use aurora_dsql_sqlx_connector::{with_retry, DsqlPool, OCCRetryConfigBuilder, Result};
use sqlx::{Connection, Row};
use std::sync::Arc;

fn build_conn_str() -> String {
    let endpoint = std::env::var("CLUSTER_ENDPOINT").expect("CLUSTER_ENDPOINT must be set");
    let user = std::env::var("CLUSTER_USER").unwrap_or_else(|_| "admin".to_string());
    format!("postgres://{}@{}/postgres", user, endpoint)
}

#[tokio::test]
#[ignore = "requires a live DSQL cluster"]
async fn test_pool_transactional_write() -> Result<()> {
    let conn_str = build_conn_str();
    let table_name = format!("tx_test_{}", std::process::id());
    let pool = DsqlPool::new(&conn_str).await?;

    // Setup
    sqlx::query(&format!(
        "CREATE TABLE IF NOT EXISTS {} (id INT PRIMARY KEY, name TEXT)",
        table_name
    ))
    .execute(&mut *pool.get().await?)
    .await
    .unwrap();

    // Manual transaction via pool connection
    {
        let mut conn = pool.get().await?;
        let mut tx = conn.begin().await.unwrap();

        sqlx::query(&format!(
            "INSERT INTO {} (id, name) VALUES (1, 'alice')",
            table_name
        ))
        .execute(&mut *tx)
        .await
        .unwrap();

        tx.commit().await.unwrap();
    }

    // Verify the write persisted
    let row = sqlx::query(&format!("SELECT name FROM {} WHERE id = 1", table_name))
        .fetch_one(&mut *pool.get().await?)
        .await
        .unwrap();
    let name: String = row.get("name");
    assert_eq!(name, "alice");

    // Cleanup
    sqlx::query(&format!("DROP TABLE {}", table_name))
        .execute(&mut *pool.get().await?)
        .await
        .unwrap();

    Ok(())
}

#[tokio::test]
#[ignore = "requires a live DSQL cluster"]
async fn test_pool_occ_concurrent_conflict() -> Result<()> {
    let conn_str = build_conn_str();
    let table_name = format!("occ_test_{}", std::process::id());
    let pool = Arc::new(DsqlPool::new(&conn_str).await?);

    // Setup: create table and seed a row
    sqlx::query(&format!(
        "CREATE TABLE IF NOT EXISTS {} (id INT PRIMARY KEY, counter INT NOT NULL)",
        table_name
    ))
    .execute(&mut *pool.get().await?)
    .await
    .unwrap();

    sqlx::query(&format!(
        "INSERT INTO {} (id, counter) VALUES (1, 0)",
        table_name
    ))
    .execute(&mut *pool.get().await?)
    .await
    .unwrap();

    let config = OCCRetryConfigBuilder::default()
        .max_attempts(5u32)
        .build()
        .unwrap();

    // Spawn two tasks that both UPDATE the same row concurrently.
    // OCC retry should allow both to eventually succeed.
    let table = table_name.clone();
    let pool1 = Arc::clone(&pool);
    let config1 = config.clone();
    let handle1 = tokio::spawn(async move {
        with_retry(&pool1, Some(&config1), |conn| {
            let t = table.clone();
            Box::pin(async move {
                sqlx::query(&format!(
                    "UPDATE {} SET counter = counter + 1 WHERE id = 1",
                    t
                ))
                .execute(conn)
                .await
                .map_err(aurora_dsql_sqlx_connector::DsqlError::DatabaseError)?;
                Ok(())
            })
        })
        .await
    });

    let table = table_name.clone();
    let pool2 = Arc::clone(&pool);
    let config2 = config.clone();
    let handle2 = tokio::spawn(async move {
        with_retry(&pool2, Some(&config2), |conn| {
            let t = table.clone();
            Box::pin(async move {
                sqlx::query(&format!(
                    "UPDATE {} SET counter = counter + 1 WHERE id = 1",
                    t
                ))
                .execute(conn)
                .await
                .map_err(aurora_dsql_sqlx_connector::DsqlError::DatabaseError)?;
                Ok(())
            })
        })
        .await
    });

    handle1.await.expect("task 1 panicked")?;
    handle2.await.expect("task 2 panicked")?;

    // Both increments should have succeeded
    let row = sqlx::query(&format!("SELECT counter FROM {} WHERE id = 1", table_name))
        .fetch_one(&mut *pool.get().await?)
        .await
        .unwrap();
    let counter: i32 = row.get("counter");
    assert_eq!(counter, 2, "Both concurrent updates should have committed");

    // Cleanup
    sqlx::query(&format!("DROP TABLE {}", table_name))
        .execute(&mut *pool.get().await?)
        .await
        .unwrap();

    Ok(())
}
