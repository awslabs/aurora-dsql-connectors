// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use aurora_dsql_sqlx_connector::DsqlPool;
use sqlx::{Connection, Executor, Row};
use std::sync::Arc;

async fn worker_task(pool: Arc<DsqlPool>, worker_id: i32) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = pool.get().await?;
    let row = sqlx::query("SELECT $1::int as value")
        .bind(worker_id)
        .fetch_one(&mut *conn)
        .await?;
    Ok(row.get("value"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cluster_endpoint =
        std::env::var("CLUSTER_ENDPOINT").expect("CLUSTER_ENDPOINT environment variable is not set");
    let cluster_user =
        std::env::var("CLUSTER_USER").unwrap_or_else(|_| "admin".to_string());

    let conn_str = format!("postgres://{}@{}/postgres", cluster_user, cluster_endpoint);

    let pool = Arc::new(DsqlPool::new(&conn_str).await?);

    // -- Concurrent read queries --
    let num_workers = 5;
    let mut handles = Vec::new();
    for i in 0..num_workers {
        let pool = Arc::clone(&pool);
        handles.push(tokio::spawn(async move { worker_task(pool, i).await }));
    }

    for handle in handles {
        let result = handle.await.expect("Worker task panicked")?;
        println!("Worker result: {}", result);
    }

    println!("Concurrent pool operations completed successfully");

    // -- Transactional write --
    {
        let mut conn = pool.get().await?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS owner(
                id uuid NOT NULL DEFAULT gen_random_uuid(),
                name varchar(30) NOT NULL,
                city varchar(80) NOT NULL,
                telephone varchar(20) DEFAULT NULL,
                PRIMARY KEY (id))",
        )
        .await?;

        let mut tx = conn.begin().await?;
        sqlx::query("INSERT INTO owner(name, city, telephone) VALUES($1, $2, $3)")
            .bind("John Doe")
            .bind("Anytown")
            .bind("555-555-1999")
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;

        let row = sqlx::query("SELECT name, city, telephone FROM owner WHERE name = $1")
            .bind("John Doe")
            .fetch_one(&mut *conn)
            .await?;

        let name: &str = row.get("name");
        let city: &str = row.get("city");
        let telephone: &str = row.get("telephone");
        println!("Inserted: name={}, city={}, telephone={}", name, city, telephone);

        // Clean up
        sqlx::query("DELETE FROM owner WHERE name = $1")
            .bind("John Doe")
            .execute(&mut *conn)
            .await?;
    }

    println!("Transactional write completed successfully");
    Ok(())
}
