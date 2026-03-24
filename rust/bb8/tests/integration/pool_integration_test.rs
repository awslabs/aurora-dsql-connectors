// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use sqlx::{Connection, Row};

use super::test_util::{build_conn_str, build_pool};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
async fn test_bb8_transactional_write() -> TestResult {
    // Use PID in table name to avoid collisions between parallel test runs
    let table_name = format!("bb8_tx_test_{}", std::process::id());
    let pool = build_pool(&build_conn_str()).await;

    // Setup
    {
        let mut conn = pool.get().await?;
        sqlx::query(&format!(
            "CREATE TABLE IF NOT EXISTS {} (id INT PRIMARY KEY, name TEXT)",
            table_name
        ))
        .execute(&mut *conn)
        .await?;
    }

    // Transactional write
    {
        let mut conn = pool.get().await?;
        let mut tx = conn.begin().await?;

        sqlx::query(&format!(
            "INSERT INTO {} (id, name) VALUES (1, 'alice')",
            table_name
        ))
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
    }

    // Verify
    {
        let mut conn = pool.get().await?;
        let row = sqlx::query(&format!("SELECT name FROM {} WHERE id = 1", table_name))
            .fetch_one(&mut *conn)
            .await?;
        let name: String = row.get("name");
        assert_eq!(name, "alice");
    }

    // Cleanup
    {
        let mut conn = pool.get().await?;
        sqlx::query(&format!("DROP TABLE {}", table_name))
            .execute(&mut *conn)
            .await?;
    }

    Ok(())
}
