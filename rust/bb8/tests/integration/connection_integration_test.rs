// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use sqlx::Row;

use super::test_util::{build_conn_str, build_pool};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
async fn test_bb8_connection() -> TestResult {
    let pool = build_pool(&build_conn_str()).await;

    let mut conn = pool.get().await?;
    let row = sqlx::query("SELECT 1 as value")
        .fetch_one(&mut *conn)
        .await?;
    let value: i32 = row.get("value");

    assert_eq!(value, 1);
    Ok(())
}
