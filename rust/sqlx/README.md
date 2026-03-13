# Aurora DSQL SQLx Connector for Rust

## Overview

A Rust connector for Amazon Aurora DSQL that wraps [SQLx](https://github.com/launchbadge/sqlx) with automatic IAM authentication. The connector handles token generation, SSL configuration, and connection management so you can focus on your application logic.

## Features

- Automatic IAM token generation (fresh token per connection)
- Connection pooling via bb8 (opt-in with `pool` feature flag)
- Single connection support for simpler use cases
- Region auto-detection from endpoint hostname
- Support for AWS profiles
- SSL always enabled with `verify-full` mode
- Connection string parsing support
- OCC retry helpers with exponential backoff and jitter

## Prerequisites

- Rust 1.75 or later
- AWS credentials configured (see [Credentials Resolution](#credentials-resolution) below)
- An Aurora DSQL cluster

For information about creating an Aurora DSQL cluster, see the [Getting started with Aurora DSQL](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/getting-started.html) guide.

### Credentials Resolution

The connector uses the [AWS SDK for Rust default credential chain](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/credproviders.html), which resolves credentials in the following order:

1. **Environment variables** (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and optionally `AWS_SESSION_TOKEN`)
2. **Shared credentials file** (`~/.aws/credentials`) with optional profile via `AWS_PROFILE` or `profile` config option
3. **Shared config file** (`~/.aws/config`)
4. **IAM role for Amazon EC2/ECS/Lambda** (instance metadata or task role)

The first source that provides valid credentials is used. You can override this by specifying `profile` in the connection string for a specific AWS profile.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
aurora-dsql-sqlx-connector = "0.0.1"
```

For connection pooling, enable the `pool` feature:

```toml
[dependencies]
aurora-dsql-sqlx-connector = { version = "0.0.1", features = ["pool"] }
```

## Configuration Options

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `host` | `Host` | (required) | Cluster endpoint |
| `region` | `Option<Region>` | (auto-detected) | AWS region |
| `user` | `User` | `User::new("admin")` | Database user |
| `database` | `String` | `"postgres"` | Database name |
| `port` | `u16` | `5432` | Database port |
| `profile` | `Option<String>` | `None` | AWS profile name for credentials |
| `token_duration_secs` | `Option<u64>` | `None` (SDK default: 900) | Token validity duration in seconds |
| `application_name` | `Option<String>` | `"aurora-dsql-rust-sqlx/{version}"` | Application name sent to Postgres |
| `pg_connect_options` | `Option<PgConnectOptions>` | `None` | Base SQLx connection options for driver-level customization |

**DsqlPoolConfig** (pool feature only) — wraps a `DsqlConfig` with pool settings:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `connection` | `DsqlConfig` | (required) | Connection configuration |
| `max_connections` | `u32` | `5` | Maximum pool connections |
| `max_lifetime_secs` | `u64` | `3300` (55 min) | Maximum connection lifetime |
| `idle_timeout_secs` | `u64` | `600` (10 min) | Maximum idle time before connection is closed |
| `occ_max_retries` | `Option<u32>` | `None` | Enable automatic OCC retry with this many attempts |

## Quick Start

```rust
use aurora_dsql_sqlx_connector::dsql_connect;
use sqlx::Row;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = dsql_connect(
        "postgres://admin@your-cluster.dsql.us-east-1.on.aws/postgres"
    ).await?;

    let row = sqlx::query("SELECT 'Hello, DSQL!' as greeting")
        .fetch_one(&mut conn)
        .await?;

    let greeting: &str = row.get("greeting");
    println!("{}", greeting);
    Ok(())
}
```

## Connection String Format

The connector supports PostgreSQL connection string format:

```
postgres://[user@]host[:port]/[database][?param=value&...]
```

Both `postgres://` and `postgresql://` schemes are supported.

**Supported query parameters:**
- `region` — AWS region
- `profile` — AWS profile name
- `tokenDurationSecs` — Token validity duration in seconds
- `maxConnections` — Maximum pool connections
- `maxLifetimeSecs` — Maximum connection lifetime in seconds
- `idleTimeoutSecs` — Maximum idle time in seconds
- `occMaxRetries` — Enable automatic OCC retry with this many attempts
- `applicationName` — Application name sent to Postgres

**Region Resolution Priority:**
1. Parse from hostname (e.g., `cluster.dsql.us-east-1.on.aws`)
2. Explicit `?region=...` in connection string
3. AWS SDK default region (`AWS_REGION` env var or `~/.aws/config`)

**Examples:**

```bash
# Full endpoint (region auto-detected from hostname)
postgres://admin@cluster.dsql.us-east-1.on.aws/postgres

# With explicit region
postgres://admin@cluster.dsql.us-east-1.on.aws/postgres?region=us-east-1

# With AWS profile
postgres://admin@cluster.dsql.us-east-1.on.aws/postgres?profile=dev

# With pool configuration
postgres://admin@cluster.dsql.us-east-1.on.aws/postgres?maxConnections=20&maxLifetimeSecs=1800
```

## Single Connection Usage

For simple scripts or when connection pooling is not needed:

```rust
use aurora_dsql_sqlx_connector::dsql_connect;
use sqlx::Row;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = dsql_connect(
        "postgres://admin@your-cluster.dsql.us-east-1.on.aws/postgres"
    ).await?;

    let row = sqlx::query("SELECT 1 as value")
        .fetch_one(&mut conn)
        .await?;
    let value: i32 = row.get("value");

    println!("Result: {}", value);
    Ok(())
}
```

Each call to `dsql_connect` or `DsqlConfig::connect` generates a fresh IAM token. For operations longer than 15 minutes, create a new connection.

## Pool Usage

Enable the `pool` feature in your `Cargo.toml`, then:

```rust
use aurora_dsql_sqlx_connector::DsqlPool;
use sqlx::Row;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = DsqlPool::new(
        "postgres://admin@your-cluster.dsql.us-east-1.on.aws/postgres"
    ).await?;

    // Get a connection from the pool
    let mut conn = pool.get().await?;

    let row = sqlx::query("SELECT 1 as value")
        .fetch_one(&mut *conn)
        .await?;
    let value: i32 = row.get("value");

    println!("Result: {}", value);
    Ok(())
}
```

The pool generates a fresh IAM token for each new connection via the bb8 `ManageConnection` hook. Token generation is a local SigV4 presigning operation (no network calls), so this adds negligible overhead.

### Pool Configuration

```rust
use aurora_dsql_sqlx_connector::{DsqlPoolConfig, DsqlPool};

let config = DsqlPoolConfig::from_connection_string(
    "postgres://admin@your-cluster.dsql.us-east-1.on.aws/postgres?\
     maxConnections=20&maxLifetimeSecs=1800&idleTimeoutSecs=300"
)?;
let pool = DsqlPool::from_config(config).await?;
```

### Transactional Writes with OCC Retry

Enable `occMaxRetries` in the pool config to opt in to automatic OCC retry. Then use `pool.with()` to run a closure inside a transaction with retry:

```rust
use aurora_dsql_sqlx_connector::{DsqlError, DsqlPool};

let pool = DsqlPool::new(
    "postgres://admin@your-cluster.dsql.us-east-1.on.aws/postgres?occMaxRetries=3"
).await?;

pool.with(|conn| {
    Box::pin(async move {
        sqlx::query("INSERT INTO items(name) VALUES($1)")
            .bind("widget")
            .execute(conn)
            .await
            .map_err(DsqlError::DatabaseError)?;
        Ok(())
    })
}).await?;
```

To opt out of retry for a specific operation, use `pool.get()` directly and manage the transaction yourself.

### Custom PgConnectOptions

For driver-level customization, provide a base `PgConnectOptions`. DSQL-required settings (host, port, user, password, database, SSL mode, application name) are always applied on top:

```rust
use aurora_dsql_sqlx_connector::{DsqlConfigBuilder, Host};
use sqlx::postgres::PgConnectOptions;

let base = PgConnectOptions::new()
    .statement_cache_capacity(500)
    .options([("search_path", "myschema")]);

let config = DsqlConfigBuilder::default()
    .host(Host::new("your-cluster.dsql.us-east-1.on.aws"))
    .pg_connect_options(Some(base))
    .build()?;

let mut conn = config.connect().await?;
```

## OCC Retry

Aurora DSQL uses optimistic concurrency control. The connector provides helpers to detect and handle OCC errors:

```rust
use aurora_dsql_sqlx_connector::{retry_on_occ, OCCRetryConfig, DsqlError};

let config = OCCRetryConfig::default(); // max_attempts: 3, exponential backoff

retry_on_occ(&config, || async {
    let mut conn = dsql_connect(
        "postgres://admin@cluster.dsql.us-east-1.on.aws/postgres"
    ).await?;
    sqlx::query("UPDATE accounts SET balance = balance - 100 WHERE id = $1")
        .bind(account_id)
        .execute(&mut conn)
        .await
        .map_err(DsqlError::DatabaseError)?;
    Ok(())
}).await?;
```

**OCC Error Detection:**
- SQLSTATE `40001` (serialization failure)
- Error codes `OC000` (data conflict) and `OC001` (schema conflict)

**Backoff Strategy:**
- Exponential backoff: `base_delay * 2^attempt`
- Additive jitter: 0–25% of delay
- Max delay: 5000ms

## Token Generation

The connector automatically generates IAM authentication tokens:

- **Connection pools**: The bb8 `ManageConnection::connect()` hook generates a fresh token for each new connection. Token generation is a local SigV4 presigning operation (no network calls), so this adds negligible overhead.
- **Single connections**: A fresh token is generated at connection time.
- **Credentials resolution**: For pools, AWS credentials are resolved once at pool creation and reused for all token generations, avoiding repeated credential chain resolution.

For the `admin` user, the connector generates admin tokens using `db_connect_admin_auth_token`. For other users, it generates standard tokens using `db_connect_auth_token`.

Token duration defaults to 15 minutes (the maximum allowed by Aurora DSQL).

## Development

### Build

```bash
# Without pool (default)
cargo build

# With pool
cargo build --features pool
```

### Run Tests

Unit tests (no cluster required):

```bash
cargo test --features pool --lib
```

Integration tests (requires a live DSQL cluster):

```bash
export CLUSTER_ENDPOINT=your-cluster.dsql.us-east-1.on.aws
cargo test --features pool --test tests
```

## Examples

The `example/` directory contains runnable examples with a standalone Cargo project:

| Example | Description |
|---------|-------------|
| [example_preferred](example/src/example_preferred.rs) | Recommended: Connection pool with concurrent queries |
| [example_no_connection_pool](example/src/alternatives/no_connection_pool/example_no_connection_pool.rs) | Single connection without pooling |

### Running Examples

```bash
export CLUSTER_ENDPOINT=your-cluster.dsql.us-east-1.on.aws
cd example

# Run the preferred example (pool-based)
cargo run --bin example_preferred

# Run the no-pool example
cargo run --bin example_no_connection_pool
```

## DSQL Best Practices

| Constraint | Recommended Approach |
|-----------|---------------------|
| No sequences / SERIAL | Use `UUID DEFAULT gen_random_uuid()` for primary keys |
| No foreign keys | Enforce referential integrity in application code |
| No TRUNCATE | Use `DELETE FROM table` |
| No extensions | No PL/pgSQL, PostGIS, pgvector |
| No triggers | Implement in application layer |
| No temp tables | Use regular tables or app-level caching |
| No SAVEPOINT | Design transactions without partial rollbacks |
| No partitioning | Manage data distribution in application |
| `CREATE INDEX ASYNC` only | Synchronous index creation is unsupported |
| Max 24 indexes/table | Max 8 columns per index |
| One DDL per transaction | Separate DDL and DML into distinct transactions |
| Transaction limits | 3,000 rows, 10 MiB, 5 minutes |
| Connection limits | 60-min max lifetime, 10,000 per cluster |
| Token expiry | 15 minutes max |
| Single database | Always `postgres` |
| Limited type system | Use VARCHAR, TEXT, INTEGER, DECIMAL, BOOLEAN, TIMESTAMP, UUID |
| Arrays/JSON as TEXT | Store as comma-separated or JSON text, cast at query time |
| Isolation level | Repeatable read (fixed) |

## Horizontal Scaling

### Connection Pool Sizing

- Start with 10–50 connections per application instance
- The pool generates fresh tokens per connection via the bb8 `ManageConnection` hook
- Respect the 10,000 max connections per cluster limit

### Batch Size Optimization

- Use batches of 500–1,000 rows (balances throughput vs. transaction limits)
- Process batches concurrently using multiple connections for bulk loading
- Smaller batches reduce lock contention and fail faster

### Hot Key Avoidance

- Always use `UUID DEFAULT gen_random_uuid()` for primary keys
- Compute aggregates via `SELECT` queries instead of maintaining running counters
- See [Avoiding Hot Keys](https://marc-bowes.com/dsql-avoid-hot-keys.html)

### Retry on Internal Errors

- Internal errors are retryable; use a new connection from the pool for the retry
- Implement backoff with jitter to avoid thundering herd

## Additional Resources

- [Amazon Aurora DSQL Documentation](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/what-is-aurora-dsql.html)
- [SQLx Documentation](https://docs.rs/sqlx/latest/sqlx/)
- [bb8 Documentation](https://docs.rs/bb8/latest/bb8/)
- [AWS SDK for Rust](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html)

---

Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.

SPDX-License-Identifier: Apache-2.0
