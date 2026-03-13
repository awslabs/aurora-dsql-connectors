// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use aurora_dsql_sqlx_connector::{
    DsqlConfigBuilder, DsqlError, DsqlPool, DsqlPoolConfigBuilder, Host, User,
};
use sqlx::{Executor, Row};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cluster_endpoint = std::env::var("CLUSTER_ENDPOINT")
        .expect("CLUSTER_ENDPOINT environment variable is not set");
    let cluster_user = std::env::var("CLUSTER_USER").unwrap_or_else(|_| "admin".to_string());

    // Build pool config with automatic OCC retry (3 attempts)
    let connection = DsqlConfigBuilder::default()
        .host(Host::new(&cluster_endpoint))
        .user(User::new(&cluster_user))
        .build()?;

    let pool_config = DsqlPoolConfigBuilder::default()
        .connection(connection)
        .occ_max_retries(Some(3))
        .build()?;

    let pool = DsqlPool::from_config(pool_config).await?;

    // -- Concurrent read queries --
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

    // -- Setup table --
    pool.get()
        .await?
        .execute(
            "CREATE TABLE IF NOT EXISTS owner(
                id uuid NOT NULL DEFAULT gen_random_uuid(),
                name varchar(30) NOT NULL,
                city varchar(80) NOT NULL,
                PRIMARY KEY (id))",
        )
        .await?;

    // -- Transactional write with automatic OCC retry via pool.with() --
    pool.with(|conn| {
        Box::pin(async move {
            sqlx::query("INSERT INTO owner(name, city) VALUES($1, $2)")
                .bind("John Doe")
                .bind("Anytown")
                .execute(conn)
                .await
                .map_err(DsqlError::DatabaseError)?;
            Ok(())
        })
    })
    .await?;

    // Verify the write
    let row = sqlx::query("SELECT name, city FROM owner WHERE name = $1")
        .bind("John Doe")
        .fetch_one(&mut *pool.get().await?)
        .await?;

    let name: &str = row.get("name");
    let city: &str = row.get("city");
    println!("Inserted: name={}, city={}", name, city);

    // Clean up
    sqlx::query("DELETE FROM owner WHERE name = $1")
        .bind("John Doe")
        .execute(&mut *pool.get().await?)
        .await?;

    println!("Transactional write completed successfully");
    Ok(())
}
