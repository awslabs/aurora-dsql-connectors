// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use aurora_dsql_sqlx_connector::{dsql_connect, Result};
use sqlx::Row;

fn build_conn_str() -> String {
    let endpoint = std::env::var("CLUSTER_ENDPOINT").expect("CLUSTER_ENDPOINT must be set");
    let user = std::env::var("CLUSTER_USER").unwrap_or_else(|_| "admin".to_string());
    format!("postgres://{}@{}/postgres", user, endpoint)
}

#[tokio::test]
#[ignore = "requires a live DSQL cluster"]
async fn test_dsql_connection() -> Result<()> {
    let conn_str = build_conn_str();

    let mut conn = dsql_connect(&conn_str).await?;

    let row = sqlx::query("SELECT 1 as value")
        .fetch_one(&mut conn)
        .await
        .unwrap();
    let value: i32 = row.get("value");

    assert_eq!(value, 1);
    Ok(())
}
