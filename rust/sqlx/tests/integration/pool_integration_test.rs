// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use aurora_dsql_sqlx_connector::{DsqlPool, Result};
use sqlx::{Connection, Row};
use std::sync::Arc;

fn build_conn_str() -> String {
    let endpoint = std::env::var("CLUSTER_ENDPOINT").expect("CLUSTER_ENDPOINT must be set");
    let user = std::env::var("CLUSTER_USER").unwrap_or_else(|_| "admin".to_string());
    format!("postgres://{}@{}/postgres", user, endpoint)
}

/// Build a pool with OCC retry configured via occMaxRetries.
async fn build_pool_with_occ(max_retries: u32) -> Result<DsqlPool> {
    let conn_str = format!("{}?occMaxRetries={}", build_conn_str(), max_retries);
    DsqlPool::new(&conn_str).await
}

#[tokio::test]
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

    // Manual transaction via pool.get() (opt-out of OCC retry)
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
async fn test_pool_occ_concurrent_conflict() -> Result<()> {
    let table_name = format!("occ_test_{}", std::process::id());
    let pool = Arc::new(build_pool_with_occ(5).await?);

    // Setup: create table and seed a row (use pool.with() so DDL is retried on OCC)
    let table = table_name.clone();
    pool.with(|conn| {
        let t = table.clone();
        Box::pin(async move {
            sqlx::query(&format!(
                "CREATE TABLE IF NOT EXISTS {} (id INT PRIMARY KEY, counter INT NOT NULL)",
                t
            ))
            .execute(&mut *conn)
            .await
            .map_err(aurora_dsql_sqlx_connector::DsqlError::DatabaseError)?;
            Ok(())
        })
    })
    .await?;

    let table = table_name.clone();
    pool.with(|conn| {
        let t = table.clone();
        Box::pin(async move {
            sqlx::query(&format!("INSERT INTO {} (id, counter) VALUES (1, 0)", t))
                .execute(&mut *conn)
                .await
                .map_err(aurora_dsql_sqlx_connector::DsqlError::DatabaseError)?;
            Ok(())
        })
    })
    .await?;

    // Spawn two tasks that both UPDATE the same row concurrently.
    // pool.with() handles OCC retry automatically via the pool config.
    let table = table_name.clone();
    let pool1 = Arc::clone(&pool);
    let handle1 = tokio::spawn(async move {
        pool1
            .with(|conn| {
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
    let handle2 = tokio::spawn(async move {
        pool2
            .with(|conn| {
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

    // Cleanup (use pool.with() so DDL is retried on OCC)
    pool.with(|conn| {
        let t = table_name.clone();
        Box::pin(async move {
            sqlx::query(&format!("DROP TABLE {}", t))
                .execute(&mut *conn)
                .await
                .map_err(aurora_dsql_sqlx_connector::DsqlError::DatabaseError)?;
            Ok(())
        })
    })
    .await?;

    Ok(())
}
