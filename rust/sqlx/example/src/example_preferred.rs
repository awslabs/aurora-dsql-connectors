// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use aurora_dsql_sqlx_connector::DsqlPool;
use sqlx::{Connection, Executor, Row};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cluster_endpoint =
        std::env::var("CLUSTER_ENDPOINT").expect("CLUSTER_ENDPOINT environment variable is not set");
    let cluster_user =
        std::env::var("CLUSTER_USER").unwrap_or_else(|_| "admin".to_string());

    let conn_str = format!("postgres://{}@{}/postgres", cluster_user, cluster_endpoint);

    let pool = DsqlPool::new(&conn_str).await?;

    // -- Concurrent read queries --
    let pool = Arc::new(pool);
    let mut handles = Vec::new();
    for i in 0..5 {
        let pool = pool.clone();
        handles.push(tokio::spawn(async move {
            let mut conn = pool.get().await?;
            let row = sqlx::query("SELECT $1::int as value")
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

    let pool = Arc::try_unwrap(pool).unwrap_or_else(|_| panic!("pool still has multiple owners"));

    // -- Transactional write --
    {
        let mut conn = pool.get().await?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS owner(
                id uuid NOT NULL DEFAULT gen_random_uuid(),
                name varchar(30) NOT NULL,
                city varchar(80) NOT NULL,
                PRIMARY KEY (id))",
        )
        .await?;

        let mut tx = conn.begin().await?;
        sqlx::query("INSERT INTO owner(name, city) VALUES($1, $2)")
            .bind("John Doe")
            .bind("Anytown")
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;

        let row = sqlx::query("SELECT name, city FROM owner WHERE name = $1")
            .bind("John Doe")
            .fetch_one(&mut *conn)
            .await?;

        let name: &str = row.get("name");
        let city: &str = row.get("city");
        println!("Inserted: name={}, city={}", name, city);

        // Clean up
        sqlx::query("DELETE FROM owner WHERE name = $1")
            .bind("John Doe")
            .execute(&mut *conn)
            .await?;
    }

    println!("Transactional write completed successfully");
    Ok(())
}
