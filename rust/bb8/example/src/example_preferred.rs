// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

//! Example: Using the Aurora DSQL connector with a bb8 connection pool.
//!
//! bb8 generates a fresh IAM auth token for each new connection via
//! ManageConnection::connect(), so no background refresh task is needed.

use aurora_dsql_bb8::DsqlConnectionManager;
use aurora_dsql_sqlx_connector::{retry_on_occ, DsqlConnectOptions, OCCRetryConfig};
use sqlx::{Connection, Row};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cluster_endpoint = std::env::var("CLUSTER_ENDPOINT")
        .expect("CLUSTER_ENDPOINT environment variable is not set");
    let cluster_user = std::env::var("CLUSTER_USER").unwrap_or_else(|_| "admin".to_string());

    let conn_str = format!("postgres://{}@{}/postgres", cluster_user, cluster_endpoint);

    let opts = DsqlConnectOptions::from_connection_string(&conn_str)?;

    let manager = DsqlConnectionManager::new(opts);
    let pool = bb8::Pool::builder()
        .max_size(5)
        .build(manager)
        .await?;

    // -- Concurrent read queries --
    let mut handles = Vec::new();
    for i in 0..5 {
        let pool = pool.clone();
        handles.push(tokio::spawn(async move {
            let mut conn = pool.get().await?;
            let row = sqlx::query("SELECT $1::int AS value")
                .bind(i)
                .fetch_one(&mut *conn)
                .await?;
            Ok::<i32, anyhow::Error>(row.get("value"))
        }));
    }

    for handle in handles {
        let result = handle.await??;
        println!("Worker result: {}", result);
    }

    println!("Concurrent pool operations completed successfully");

    // -- Setup table --
    {
        let mut conn = pool.get().await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS owner(
                id uuid NOT NULL DEFAULT gen_random_uuid(),
                name varchar(30) NOT NULL,
                city varchar(80) NOT NULL,
                PRIMARY KEY (id))",
        )
        .execute(&mut *conn)
        .await?;
    }

    // -- Transactional write with OCC retry --
    let occ_config = OCCRetryConfig::default();

    retry_on_occ(&occ_config, || {
        let pool = pool.clone();
        async move {
            let mut conn = pool.get().await.map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
            let mut tx = conn.begin().await?;

            sqlx::query("INSERT INTO owner(name, city) VALUES($1, $2)")
                .bind("John Doe")
                .bind("Anytown")
                .execute(&mut *tx)
                .await?;

            tx.commit().await?;
            Ok(())
        }
    })
    .await?;

    // Verify the write
    {
        let mut conn = pool.get().await?;
        let row = sqlx::query("SELECT name, city FROM owner WHERE name = $1")
            .bind("John Doe")
            .fetch_one(&mut *conn)
            .await?;

        let name: &str = row.get("name");
        let city: &str = row.get("city");
        println!("Inserted: name={}, city={}", name, city);
    }

    // Clean up with OCC retry
    retry_on_occ(&occ_config, || {
        let pool = pool.clone();
        async move {
            let mut conn = pool.get().await.map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
            let mut tx = conn.begin().await?;

            sqlx::query("DELETE FROM owner WHERE name = $1")
                .bind("John Doe")
                .execute(&mut *tx)
                .await?;

            tx.commit().await?;
            Ok(())
        }
    })
    .await?;

    println!("Transactional write completed successfully");

    Ok(())
}
