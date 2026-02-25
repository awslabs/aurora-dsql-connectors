use aurora_dsql_sqlx_connector::{DsqlPool, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let conn_str = std::env::var("DSQL_CONNECTION_STRING")
        .expect("DSQL_CONNECTION_STRING environment variable not set");

    let pool = DsqlPool::new(&conn_str).await?;

    // Test 1: Simple SELECT
    println!("Test 1: Simple SELECT");
    let result: (i32,) = pool.fetch_one("SELECT 1").await?;
    println!("  Result: {}\n", result.0);

    // Test 2: CREATE TABLE
    println!("Test 2: CREATE TABLE");
    pool.execute("CREATE TABLE IF NOT EXISTS test_users (id INT PRIMARY KEY, name TEXT, age INT)")
        .await?;
    println!("  Table created\n");

    // Test 3: INSERT
    println!("Test 3: INSERT");
    pool.execute(
        "INSERT INTO test_users (id, name, age) VALUES (1, 'Ajmeer', 30), (2, 'Ragul', 25)",
    )
    .await?;
    println!("  Rows inserted\n");

    // Test 4: SELECT with multiple rows
    println!("Test 4: SELECT multiple rows");
    let users: Vec<(i32, String, i32)> = pool
        .fetch_all("SELECT id, name, age FROM test_users ORDER BY id")
        .await?;
    for (id, name, age) in users {
        println!("  User: id={}, name={}, age={}", id, name, age);
    }
    println!();

    // Test 5: UPDATE
    println!("Test 5: UPDATE");
    pool.execute("UPDATE test_users SET age = 31 WHERE name = 'Ajmeer'")
        .await?;
    println!("  Row updated\n");

    // Test 6: Verify UPDATE
    println!("Test 6: Verify UPDATE");
    let age: (i32,) = pool
        .fetch_one("SELECT age FROM test_users WHERE name = 'Ajmeer'")
        .await?;
    println!("  Ajmeer's age: {}\n", age.0);

    // Test 7: DELETE
    println!("Test 7: DELETE");
    pool.execute("DELETE FROM test_users WHERE name = 'Ragul'")
        .await?;
    println!("  Row deleted\n");

    // Test 8: COUNT
    println!("Test 8: COUNT");
    let count: (i64,) = pool.fetch_one("SELECT COUNT(*) FROM test_users").await?;
    println!("  Remaining users: {}\n", count.0);

    // Test 9: DROP TABLE
    println!("Test 9: DROP TABLE");
    pool.execute("DROP TABLE test_users").await?;
    println!("  Table dropped\n");

    println!("All tests passed! ✅");
    Ok(())
}
