// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use aurora_dsql_sqlx_connector::{DsqlConnection, Result};
use sqlx::Row;

fn build_conn_str() -> Option<String> {
    let endpoint = std::env::var("CLUSTER_ENDPOINT").ok()?;
    let user = std::env::var("CLUSTER_USER").unwrap_or_else(|_| "admin".to_string());
    Some(format!("postgres://{}@{}/postgres", user, endpoint))
}

#[tokio::test]
async fn test_dsql_connection() -> Result<()> {
    let conn_str = match build_conn_str() {
        Some(v) => v,
        None => {
            eprintln!("CLUSTER_ENDPOINT not set, skipping integration test");
            return Ok(());
        }
    };

    let mut conn = DsqlConnection::connect_with(&conn_str).await?;

    let row = sqlx::query("SELECT 1 as value")
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    let value: i32 = row.get("value");

    assert_eq!(value, 1);
    Ok(())
}