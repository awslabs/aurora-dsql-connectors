# Aurora DSQL bb8 Connection Pool for Rust

## Overview

A [bb8](https://docs.rs/bb8) connection pool integration for Amazon Aurora DSQL. This crate implements `bb8::ManageConnection` on top of the [Aurora DSQL SQLx Connector](../sqlx/), generating a fresh IAM auth token for every new connection.

Use this when you prefer bb8's connection pool over sqlx's built-in pool.

## Features

- Fresh IAM auth token generated per connection — no background refresh needed
- Built on top of the [Aurora DSQL SQLx Connector](../sqlx/) (`DsqlConnectOptions`)
- Connection health checks via ping
- Admin and regular user token support (auto-detected from username)
- SSL always enabled with `verify-full` mode
- OCC retry support via the SQLx connector's `occ` feature

## Prerequisites

- Rust 1.80 or later
- AWS credentials configured (see [Credentials Resolution](../sqlx/README.md#credentials-resolution))
- An Aurora DSQL cluster

For information about creating an Aurora DSQL cluster, see the [Getting started with Aurora DSQL](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/getting-started.html) guide.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
aurora-dsql-bb8 = "0.0.1"
aurora-dsql-sqlx-connector = { version = "0.0.1", features = ["occ"] }
bb8 = "0.9"
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres"] }
```

## Quick Start

```rust
use aurora_dsql_bb8::DsqlConnectionManager;
use aurora_dsql_sqlx_connector::DsqlConnectOptions;
use sqlx::Row;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts = DsqlConnectOptions::from_connection_string(
        "postgres://admin@your-cluster.dsql.us-east-1.on.aws/postgres"
    )?;

    let manager = DsqlConnectionManager::new(opts);
    let pool = bb8::Pool::builder()
        .max_size(5)
        .build(manager)
        .await?;

    let mut conn = pool.get().await?;
    let row = sqlx::query("SELECT 'Hello, DSQL!' as greeting")
        .fetch_one(&mut *conn)
        .await?;

    let greeting: &str = row.get("greeting");
    println!("{}", greeting);

    Ok(())
}
```

## Token Behavior

Each call to `bb8::ManageConnection::connect()` generates a fresh IAM auth token. This means no background refresh task is needed — tokens are always current when a new connection is created.

Token generation is a local SigV4 presigning operation with negligible cost. For the `admin` user, the connector generates admin tokens; for other users, it generates standard tokens.

## Pool Configuration

Customize the bb8 pool for Aurora DSQL's connection characteristics:

```rust
let manager = DsqlConnectionManager::new(opts);
let pool = bb8::Pool::builder()
    .max_size(10)
    .min_idle(Some(2))
    .max_lifetime(Some(std::time::Duration::from_secs(3300))) // under DSQL's 60-min limit
    .idle_timeout(Some(std::time::Duration::from_secs(600)))
    .connection_timeout(std::time::Duration::from_secs(30))
    .build(manager)
    .await?;
```

Key settings for DSQL:
- **`max_lifetime`**: Keep under 3600s (DSQL terminates connections at 60 minutes)
- **`max_size`**: More concurrent connections with smaller batches yields better throughput
- **`idle_timeout`**: Reclaim idle connections to avoid stale connections

## Configuration

Connection configuration is handled by `DsqlConnectOptions` from the SQLx connector crate. See the [SQLx connector README](../sqlx/README.md) for details on:

- [Connection string format](../sqlx/README.md#connection-string-format)
- [Configuration options](../sqlx/README.md#configuration-options)
- [OCC retry helpers](../sqlx/README.md#occ-retry)
- [Additional resources](../sqlx/README.md#additional-resources)

## Examples

The `example/` directory contains a runnable example with a standalone Cargo project:

| Example | Description |
|---------|-------------|
| [example_preferred](example/src/example_preferred.rs) | Pool with concurrent queries, OCC retry, and transactional writes |

### Running the Example

```bash
export CLUSTER_ENDPOINT=your-cluster.dsql.us-east-1.on.aws
cd example
cargo run --bin example_preferred
```

## Development

### Build

```bash
cargo build
```

### Run Tests

Unit tests (no cluster required):

```bash
cargo test --lib
```

Integration tests (requires a live DSQL cluster):

```bash
export CLUSTER_ENDPOINT=your-cluster.dsql.us-east-1.on.aws
cargo test --test tests
```

## Additional Resources

- [Amazon Aurora DSQL Documentation](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/what-is-aurora-dsql.html)
- [bb8 Documentation](https://docs.rs/bb8)
- [Aurora DSQL SQLx Connector](../sqlx/)

---

Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.

SPDX-License-Identifier: Apache-2.0
