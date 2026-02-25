use aurora_dsql_sqlx_connector::{DsqlPool, Result};
use sqlx::Row;

#[tokio::test]
async fn test_basic_connection() -> Result<()> {
    let conn_str =
        std::env::var("DSQL_CONNECTION_STRING").expect("DSQL_CONNECTION_STRING required");

    let pool = DsqlPool::new(&conn_str).await?;

    let row = sqlx::query("SELECT 1 as value")
        .fetch_one(&*pool)
        .await
        .unwrap();
    let value: i32 = row.get("value");

    assert_eq!(value, 1);
    Ok(())
}

#[tokio::test]
async fn test_query_execution() -> Result<()> {
    let conn_str =
        std::env::var("DSQL_CONNECTION_STRING").expect("DSQL_CONNECTION_STRING required");

    let pool = DsqlPool::new(&conn_str).await?;

    sqlx::query("CREATE TABLE IF NOT EXISTS hydra_test (id INT PRIMARY KEY, data TEXT)")
        .execute(&*pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO hydra_test (id, data) VALUES (1, 'test')")
        .execute(&*pool)
        .await
        .unwrap();

    let row = sqlx::query("SELECT data FROM hydra_test WHERE id = 1")
        .fetch_one(&*pool)
        .await
        .unwrap();
    let data: String = row.get("data");

    assert_eq!(data, "test");

    sqlx::query("DROP TABLE hydra_test")
        .execute(&*pool)
        .await
        .unwrap();

    Ok(())
}
