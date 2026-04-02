// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for trait-based OCC retry.
//!
//! These tests require a running Aurora DSQL cluster with credentials configured.
//! Set CLUSTER_ENDPOINT environment variable to run these tests.

use aurora_dsql_sqlx_connector::{
    is_occ_error, pool, retry_on_occ, OCCRetryConfig, OCCRetryConfigBuilder, Result, RetryExecutor,
};
use sqlx::{Acquire, Row};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use super::test_util::build_conn_str;

#[tokio::test]
async fn test_journey_1_default_config_pgpool() -> Result<()> {
    // Journey 1: sqlx pool + default config (simplest)
    let url = build_conn_str();
    let table_name = format!("test_j1_{}", std::process::id());

    let pool = pool::connect(&url).await?;

    let occ_config = OCCRetryConfigBuilder::default()
        .max_attempts(10)
        .base_delay_ms(100)
        .build()
        .unwrap();

    // Create test table with OCC retry in transaction (handles schema propagation delays)
    retry_on_occ(&occ_config, || {
        let pool = pool.clone();
        let t = table_name.clone();
        async move {
            let mut conn = pool.acquire().await?;
            let mut tx = conn.begin().await?;
            sqlx::query(&format!(
                "CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY, value TEXT)",
                t
            ))
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            Ok(())
        }
    })
    .await?;

    // Insert with automatic retry using trait
    pool.query(&format!(
        "INSERT INTO {} (id, value) VALUES ($1, $2)",
        table_name
    ))
    .bind(1)
    .bind("alice")
    .execute()
    .await?;

    // Fetch with automatic retry
    let row = pool
        .query(&format!("SELECT value FROM {} WHERE id = $1", table_name))
        .bind(1)
        .fetch_one()
        .await?;

    let value: String = row.get("value");
    assert_eq!(value, "alice");

    // Cleanup
    pool.query(&format!("DROP TABLE {}", table_name))
        .execute()
        .await?;

    pool.close().await;
    Ok(())
}

#[tokio::test]

async fn test_journey_2_custom_config_retry_pool() -> Result<()> {
    // Journey 2: sqlx pool + custom config
    let url = build_conn_str();
    let table_name = format!("test_j2_{}", std::process::id());

    let config = OCCRetryConfigBuilder::default()
        .max_attempts(10)
        .base_delay_ms(100)
        .max_delay_ms(2000)
        .build()
        .unwrap();

    let pool = pool::connect_with_retry(&url, config.clone()).await?;

    // Verify config is stored
    assert_eq!(pool.config().max_attempts(), 10);
    assert_eq!(pool.config().base_delay_ms(), 100);

    // Create test table with OCC retry in transaction
    retry_on_occ(&config, || {
        let inner_pool = pool.inner().clone();
        let t = table_name.clone();
        async move {
            let mut conn = inner_pool.acquire().await?;
            let mut tx = conn.begin().await?;
            sqlx::query(&format!(
                "CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY, balance INTEGER)",
                t
            ))
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            Ok(())
        }
    })
    .await?;

    pool.query(&format!(
        "INSERT INTO {} (id, balance) VALUES (1, 1000)",
        table_name
    ))
    .execute()
    .await?;

    // Update with custom retry config
    pool.query(&format!(
        "UPDATE {} SET balance = balance - $1 WHERE id = $2",
        table_name
    ))
    .bind(100)
    .bind(1)
    .execute()
    .await?;

    // Verify result
    let row = pool
        .query(&format!("SELECT balance FROM {} WHERE id = 1", table_name))
        .fetch_one()
        .await?;

    let balance: i32 = row.get("balance");
    assert_eq!(balance, 900);

    // Test .begin() delegation with retry wrapper for schema propagation
    retry_on_occ(&config, || {
        let inner_pool = pool.inner().clone();
        let t = table_name.clone();
        async move {
            let mut conn = inner_pool.acquire().await?;
            let mut tx = conn.begin().await?;
            sqlx::query(&format!("INSERT INTO {} (id, balance) VALUES (2, 500)", t))
                .execute(&mut *tx)
                .await?;
            tx.commit().await
        }
    })
    .await?;

    // Cleanup
    pool.query(&format!("DROP TABLE {}", table_name))
        .execute()
        .await?;

    pool.close().await;
    Ok(())
}

#[tokio::test]

async fn test_journey_3_per_query_opt_out() -> Result<()> {
    // Journey 3: Per-query opt-out
    let url = build_conn_str();
    let table_name = format!("test_j3_{}", std::process::id());

    let pool = pool::connect(&url).await?;

    let occ_config = OCCRetryConfigBuilder::default()
        .max_attempts(10)
        .base_delay_ms(100)
        .build()
        .unwrap();

    // Create test table with OCC retry in transaction
    retry_on_occ(&occ_config, || {
        let pool = pool.clone();
        let t = table_name.clone();
        async move {
            let mut conn = pool.acquire().await?;
            let mut tx = conn.begin().await?;
            sqlx::query(&format!(
                "CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY, data TEXT)",
                t
            ))
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            Ok(())
        }
    })
    .await?;

    // Write with retry (default)
    pool.query(&format!(
        "INSERT INTO {} (id, data) VALUES ($1, $2)",
        table_name
    ))
    .bind(1)
    .bind("event_data")
    .execute()
    .await?;

    // Read without retry (opt-out)
    let row = pool
        .query(&format!("SELECT COUNT(*) as total FROM {}", table_name))
        .without_retry()
        .fetch_one()
        .await?;

    let count: i64 = row.get("total");
    assert_eq!(count, 1);

    // Cleanup
    pool.query(&format!("DROP TABLE {}", table_name))
        .execute()
        .await?;

    pool.close().await;
    Ok(())
}

#[tokio::test]

async fn test_journey_5_single_connection_with_retry_on_occ() -> Result<()> {
    // Journey 5: Single connection with retry_on_occ
    let url = build_conn_str();

    let config = OCCRetryConfig::default();

    // Create table outside retry closure
    let pool = pool::connect(&url).await?;
    pool.query("DROP TABLE IF EXISTS test_journey_5")
        .execute()
        .await
        .ok();
    pool.query("CREATE TABLE test_journey_5 (id INTEGER PRIMARY KEY, value TEXT)")
        .execute()
        .await?;
    pool.close().await;

    // Use retry_on_occ with connection::connect
    retry_on_occ(&config, || async {
        let mut conn = aurora_dsql_sqlx_connector::connection::connect(&url)
            .await
            .map_err(|e| match e {
                aurora_dsql_sqlx_connector::DsqlError::ConnectionError(sqlx_err) => sqlx_err,
                other => sqlx::Error::Protocol(other.to_string().into()),
            })?;

        sqlx::query("INSERT INTO test_journey_5 (id, value) VALUES ($1, $2)")
            .bind(1)
            .bind("test")
            .execute(&mut conn)
            .await
    })
    .await?;

    // Verify insertion
    let pool2 = pool::connect(&url).await?;
    let row = pool2
        .query("SELECT value FROM test_journey_5 WHERE id = 1")
        .fetch_one()
        .await?;

    let value: String = row.get("value");
    assert_eq!(value, "test");

    // Cleanup
    pool2.query("DROP TABLE test_journey_5").execute().await?;
    pool2.close().await;
    Ok(())
}

#[tokio::test]

async fn test_parameter_replay_across_retries() -> Result<()> {
    let url = build_conn_str();
    let table_name = format!("test_params_{}", std::process::id());

    let occ_config = OCCRetryConfigBuilder::default()
        .max_attempts(10)
        .base_delay_ms(100)
        .build()
        .unwrap();

    let pool = pool::connect_with_retry(&url, occ_config.clone()).await?;

    // Create test table with OCC retry in transaction
    retry_on_occ(&occ_config, || {
        let inner_pool = pool.inner().clone();
        let t = table_name.clone();
        async move {
            let mut conn = inner_pool.acquire().await?;
            let mut tx = conn.begin().await?;
            sqlx::query(&format!(
                "CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)",
                t
            ))
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            Ok(())
        }
    })
    .await?;

    // Insert with multiple parameters using automatic retry
    pool.query(&format!(
        "INSERT INTO {} (id, name, age) VALUES ($1, $2, $3)",
        table_name
    ))
    .bind(1)
    .bind("alice")
    .bind(30)
    .execute()
    .await?;

    // Verify all parameters were correctly bound
    let row = pool
        .query(&format!(
            "SELECT name, age FROM {} WHERE id = 1",
            table_name
        ))
        .fetch_one()
        .await?;

    let name: String = row.get("name");
    let age: i32 = row.get("age");
    assert_eq!(name, "alice");
    assert_eq!(age, 30);

    // Cleanup
    pool.query(&format!("DROP TABLE {}", table_name))
        .execute()
        .await?;

    pool.close().await;
    Ok(())
}

#[tokio::test]

async fn test_fetch_all_with_retry() -> Result<()> {
    let url = build_conn_str();
    let table_name = format!("test_fetch_all_{}", std::process::id());

    let occ_config = OCCRetryConfigBuilder::default()
        .max_attempts(10)
        .base_delay_ms(100)
        .build()
        .unwrap();

    let pool = pool::connect_with_retry(&url, occ_config.clone()).await?;

    // Create test table with OCC retry in transaction
    retry_on_occ(&occ_config, || {
        let pool = pool.clone();
        let t = table_name.clone();
        async move {
            let mut conn = pool.acquire().await?;
            let mut tx = conn.begin().await?;
            sqlx::query(&format!(
                "CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY, value TEXT)",
                t
            ))
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            Ok(())
        }
    })
    .await?;

    // Insert multiple rows
    for i in 1..=5 {
        pool.query(&format!(
            "INSERT INTO {} (id, value) VALUES ($1, $2)",
            table_name
        ))
        .bind(i)
        .bind(format!("value_{}", i))
        .execute()
        .await?;
    }

    // Fetch all rows
    let rows = pool
        .query(&format!("SELECT id, value FROM {} ORDER BY id", table_name))
        .fetch_all()
        .await?;

    assert_eq!(rows.len(), 5);
    for (idx, row) in rows.iter().enumerate() {
        let id: i32 = row.get("id");
        let value: String = row.get("value");
        assert_eq!(id, (idx + 1) as i32);
        assert_eq!(value, format!("value_{}", idx + 1));
    }

    // Cleanup
    pool.query(&format!("DROP TABLE {}", table_name))
        .execute()
        .await?;

    pool.close().await;
    Ok(())
}

#[tokio::test]

async fn test_fetch_optional_with_retry() -> Result<()> {
    let url = build_conn_str();
    let table_name = format!("test_fetch_opt_{}", std::process::id());

    let pool = pool::connect(&url).await?;

    let occ_config = OCCRetryConfigBuilder::default()
        .max_attempts(10)
        .base_delay_ms(100)
        .build()
        .unwrap();

    // Create test table with OCC retry in transaction
    retry_on_occ(&occ_config, || {
        let pool = pool.clone();
        let t = table_name.clone();
        async move {
            let mut conn = pool.acquire().await?;
            let mut tx = conn.begin().await?;
            sqlx::query(&format!(
                "CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY, value TEXT)",
                t
            ))
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            Ok(())
        }
    })
    .await?;

    // Fetch from empty table (should return None)
    let result = pool
        .query(&format!("SELECT value FROM {} WHERE id = 1", table_name))
        .fetch_optional()
        .await?;

    assert!(result.is_none());

    // Insert a row
    pool.query(&format!(
        "INSERT INTO {} (id, value) VALUES (1, 'exists')",
        table_name
    ))
    .execute()
    .await?;

    // Fetch again (should return Some)
    let result = pool
        .query(&format!("SELECT value FROM {} WHERE id = 1", table_name))
        .fetch_optional()
        .await?;

    assert!(result.is_some());
    let row = result.unwrap();
    let value: String = row.get("value");
    assert_eq!(value, "exists");

    // Cleanup
    pool.query(&format!("DROP TABLE {}", table_name))
        .execute()
        .await?;

    pool.close().await;
    Ok(())
}

#[tokio::test]

async fn test_actual_occ_retry() -> Result<()> {
    // This test simulates an actual OCC conflict and verifies retry behavior.
    // NOTE: Creating reliable OCC conflicts in tests is challenging.
    // This test uses a counter to verify retry logic is invoked.

    let url = build_conn_str();
    let table_name = format!("test_occ_{}", std::process::id());

    let occ_config = OCCRetryConfigBuilder::default()
        .max_attempts(10)
        .base_delay_ms(100)
        .build()
        .unwrap();

    let pool = pool::connect_with_retry(&url, occ_config.clone()).await?;

    // Create test table with OCC retry in transaction
    retry_on_occ(&occ_config, || {
        let pool = pool.clone();
        let t = table_name.clone();
        async move {
            let mut conn = pool.acquire().await?;
            let mut tx = conn.begin().await?;
            sqlx::query(&format!(
                "CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY, counter INTEGER)",
                t
            ))
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            Ok(())
        }
    })
    .await?;

    retry_on_occ(&occ_config, || {
        let pool = pool.clone();
        let t = table_name.clone();
        async move {
            let mut conn = pool.acquire().await?;
            let mut tx = conn.begin().await?;
            sqlx::query(&format!("INSERT INTO {} (id, counter) VALUES (1, 0)", t))
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            Ok(())
        }
    })
    .await?;

    // Simulate OCC conflict with concurrent transactions
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let config = OCCRetryConfigBuilder::default()
        .max_attempts(5)
        .base_delay_ms(10)
        .build()
        .unwrap();

    // Use retry_on_occ to wrap a transaction that might conflict
    let result = retry_on_occ(&config, || {
        let counter = counter_clone.clone();
        let pool = pool.clone();
        let t = table_name.clone();
        async move {
            counter.fetch_add(1, Ordering::SeqCst);

            let mut tx = pool.begin().await?;
            sqlx::query(&format!(
                "UPDATE {} SET counter = counter + 1 WHERE id = 1",
                t
            ))
            .execute(&mut *tx)
            .await?;
            tx.commit().await
        }
    })
    .await;

    // Should either succeed or exhaust retries with OCC error
    // (Parallel test execution can cause real OCC conflicts)
    match result {
        Ok(_) => {
            // Success - verify at least one attempt was made
            assert!(counter.load(Ordering::SeqCst) >= 1);
        }
        Err(aurora_dsql_sqlx_connector::DsqlError::OCCRetryExhausted { attempts, .. }) => {
            // OCC retry exhausted - verify we tried the configured number of times
            assert_eq!(attempts, 5);
            assert_eq!(counter.load(Ordering::SeqCst), 5);
        }
        Err(e) => panic!("Unexpected error: {:?}", e),
    }

    // Cleanup
    pool.query(&format!("DROP TABLE {}", table_name))
        .execute()
        .await?;

    pool.close().await;
    Ok(())
}

#[tokio::test]

async fn test_is_occ_error_detection() -> Result<()> {
    // Test that is_occ_error correctly identifies OCC errors
    // This is more of a unit test but included for completeness

    let url = build_conn_str();

    let pool = pool::connect(&url).await?;

    // Create test table with constraint to trigger non-OCC error
    pool.query("DROP TABLE IF EXISTS test_error_detection")
        .execute()
        .await
        .ok();

    pool.query("CREATE TABLE test_error_detection (id INTEGER PRIMARY KEY)")
        .execute()
        .await?;

    pool.query("INSERT INTO test_error_detection (id) VALUES (1)")
        .execute()
        .await?;

    // Try to insert duplicate - should get unique violation, not OCC error
    let result = pool
        .query("INSERT INTO test_error_detection (id) VALUES (1)")
        .execute()
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();

    // This should NOT be an OCC error (it's a unique constraint violation)
    match err {
        aurora_dsql_sqlx_connector::DsqlError::DatabaseError(sqlx_err) => {
            assert!(!is_occ_error(&sqlx_err));
        }
        _ => panic!("Expected DatabaseError"),
    }

    // Cleanup
    pool.query("DROP TABLE test_error_detection")
        .execute()
        .await?;

    pool.close().await;
    Ok(())
}
