// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use aurora_dsql_sqlx_connector::{DsqlPool, Result};
use sqlx::Row;

fn build_conn_str() -> Option<String> {
    let endpoint = std::env::var("CLUSTER_ENDPOINT").ok()?;
    let user = std::env::var("CLUSTER_USER").unwrap_or_else(|_| "admin".to_string());
    Some(format!("postgres://{}@{}/postgres", user, endpoint))
}

#[tokio::test]
async fn test_pool_basic_connection() -> Result<()> {
    let conn_str = match build_conn_str() {
        Some(v) => v,
        None => {
            eprintln!("CLUSTER_ENDPOINT not set, skipping integration test");
            return Ok(());
        }
    };

    let pool = DsqlPool::new(&conn_str).await?;

    let row = sqlx::query("SELECT 1 as value")
        .fetch_one(&mut *pool.get().await?)
        .await
        .unwrap();
    let value: i32 = row.get("value");

    assert_eq!(value, 1);
    Ok(())
}

#[tokio::test]
async fn test_pool_query_execution() -> Result<()> {
    let conn_str = match build_conn_str() {
        Some(v) => v,
        None => {
            eprintln!("CLUSTER_ENDPOINT not set, skipping integration test");
            return Ok(());
        }
    };

    let table_name = format!("pool_test_{}", std::process::id());
    let pool = DsqlPool::new(&conn_str).await?;

    sqlx::query(&format!(
        "CREATE TABLE IF NOT EXISTS {} (id INT PRIMARY KEY, data TEXT)",
        table_name
    ))
    .execute(&mut *pool.get().await?)
    .await
    .unwrap();

    sqlx::query(&format!(
        "INSERT INTO {} (id, data) VALUES (1, 'test')",
        table_name
    ))
    .execute(&mut *pool.get().await?)
    .await
    .unwrap();

    let row = sqlx::query(&format!(
        "SELECT data FROM {} WHERE id = 1",
        table_name
    ))
    .fetch_one(&mut *pool.get().await?)
    .await
    .unwrap();
    let data: String = row.get("data");

    assert_eq!(data, "test");

    sqlx::query(&format!("DROP TABLE {}", table_name))
        .execute(&mut *pool.get().await?)
        .await
        .unwrap();

    Ok(())
}
