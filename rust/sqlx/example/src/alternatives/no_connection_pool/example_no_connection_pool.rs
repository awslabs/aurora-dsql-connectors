// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use aurora_dsql_sqlx_connector::dsql_connect;
use sqlx::{Executor, Row};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cluster_endpoint = std::env::var("CLUSTER_ENDPOINT")
        .expect("CLUSTER_ENDPOINT environment variable is not set");
    let cluster_user = std::env::var("CLUSTER_USER").unwrap_or_else(|_| "admin".to_string());

    let conn_str = format!("postgres://{}@{}/postgres", cluster_user, cluster_endpoint);

    let mut conn = dsql_connect(&conn_str).await?;

    // Create table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS owner(
            id uuid NOT NULL DEFAULT gen_random_uuid(),
            name varchar(30) NOT NULL,
            city varchar(80) NOT NULL,
            PRIMARY KEY (id))",
    )
    .await?;

    // Insert a row
    conn.execute("INSERT INTO owner(name, city) VALUES('John Doe', 'Anytown')")
        .await?;

    // Query it back
    let row = sqlx::query("SELECT * FROM owner WHERE name = $1")
        .bind("John Doe")
        .fetch_one(&mut conn)
        .await?;

    let name: &str = row.get("name");
    let city: &str = row.get("city");
    println!("name={}, city={}", name, city);

    assert_eq!(name, "John Doe");
    assert_eq!(city, "Anytown");

    // Clean up
    sqlx::query("DELETE FROM owner WHERE name = $1")
        .bind("John Doe")
        .execute(&mut conn)
        .await?;

    println!("Connection exercised successfully");
    Ok(())
}
