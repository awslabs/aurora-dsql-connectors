# Aurora DSQL pgx Connector for Go

## Overview

A Go connector for Amazon Aurora DSQL that wraps [pgx](https://github.com/jackc/pgx) with automatic IAM authentication. The connector handles token generation, SSL configuration, and connection management so you can focus on your application logic.

## Features

- Automatic IAM token generation
- Connection pooling via `pgxpool`
- Single connection support for simpler use cases
- Flexible host configuration (full endpoint or cluster ID)
- Region auto-detection from endpoint hostname
- Support for AWS profiles and custom credentials providers
- SSL always enabled with `verify-full` mode and direct TLS negotiation
- Connection string parsing support

## Prerequisites

- Go 1.24 or later
- AWS credentials configured (see [Credentials Resolution](#credentials-resolution) below)
- An Aurora DSQL cluster

For information about creating an Aurora DSQL cluster, see the [Getting started with Aurora DSQL](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/getting-started.html) guide.

### Credentials Resolution

The connector uses the [AWS SDK for Go v2 default credential chain](https://aws.github.io/aws-sdk-go-v2/docs/configuring-sdk/#specifying-credentials), which resolves credentials in the following order:

1. **Environment variables** (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and optionally `AWS_SESSION_TOKEN`)
2. **Shared credentials file** (`~/.aws/credentials`) with optional profile via `AWS_PROFILE` or `Config.Profile`
3. **Shared config file** (`~/.aws/config`)
4. **IAM role for Amazon EC2/ECS/Lambda** (instance metadata or task role)

The first source that provides valid credentials is used. You can override this by specifying `Config.Profile` for a specific AWS profile or `Config.CustomCredentialsProvider` for complete control over credential resolution.

## Installation

```bash
go get github.com/awslabs/aurora-dsql-connectors/go/pgx/dsql
```

## Configuration Options

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `Host` | `string` | (required) | Cluster endpoint or cluster ID |
| `Region` | `string` | (auto-detected) | AWS region; required if Host is a cluster ID |
| `User` | `string` | `"admin"` | Database user |
| `Database` | `string` | `"postgres"` | Database name |
| `Port` | `int` | `5432` | Database port |
| `Profile` | `string` | `""` | AWS profile name for credentials |
| `TokenDurationSecs` | `int` | `900` (15 min) | Token validity duration in seconds (max 1 week) |
| `CustomCredentialsProvider` | `aws.CredentialsProvider` | `nil` | Custom AWS credentials provider |

Pool configuration is passed directly via `*pgxpool.Config` as a separate parameter to `NewPool`. See [Pool Configuration Tuning](#pool-configuration-tuning) for details.

## Quick Start

```go
package main

import (
    "context"
    "log"

    "github.com/awslabs/aurora-dsql-connectors/go/pgx/dsql"
)

func main() {
    ctx := context.Background()

    // Create a connection pool
    pool, err := dsql.NewPool(ctx, dsql.Config{
        Host: "a1b2c3d4e5f6g7h8i9j0klmnop.dsql.us-east-1.on.aws",
    })
    if err != nil {
        log.Fatal(err)
    }
    defer pool.Close()

    // Execute a query
    var greeting string
    err = pool.QueryRow(ctx, "SELECT 'Hello, DSQL!'").Scan(&greeting)
    if err != nil {
        log.Fatal(err)
    }
    log.Println(greeting)
}
```

## Connection String Format

The connector supports PostgreSQL connection string format:

```
postgres://[user@]host[:port]/[database][?param=value&...]
```

**Supported query parameters:**
- `region` - AWS region
- `profile` - AWS profile name
- `tokenDurationSecs` - Token validity duration in seconds

**Examples:**

```go
// Full endpoint (region auto-detected)
pool, _ := dsql.NewPool(ctx, "postgres://admin@cluster.dsql.us-east-1.on.aws/postgres")

// With explicit region
pool, _ := dsql.NewPool(ctx, "postgres://admin@cluster.dsql.us-east-1.on.aws/mydb?region=us-east-1")

// With AWS profile
pool, _ := dsql.NewPool(ctx, "postgres://admin@cluster.dsql.us-east-1.on.aws/postgres?profile=dev")
```

## Advanced Usage

### Host Configuration

The connector supports two host formats:

**Full endpoint** (region auto-detected):
```go
pool, _ := dsql.NewPool(ctx, dsql.Config{
    Host: "a1b2c3d4e5f6g7h8i9j0klmnop.dsql.us-east-1.on.aws",
})
```

**Cluster ID** (region required):
```go
pool, _ := dsql.NewPool(ctx, dsql.Config{
    Host:   "a1b2c3d4e5f6g7h8i9j0klmnop",
    Region: "us-east-1",
})
```

If using a cluster ID, the region can also be set via `AWS_REGION` or `AWS_DEFAULT_REGION` environment variables.

### Custom Credentials Provider

For cross-account access or other credential scenarios:

```go
// Create an assume-role credentials provider
credsProvider, err := dsql.NewAssumeRoleCredentialsProvider(
    ctx,
    "arn:aws:iam::123456789012:role/DSQLAccessRole",
    "us-east-1",
)
if err != nil {
    log.Fatal(err)
}

pool, err := dsql.NewPool(ctx, dsql.Config{
    Host:                      "a1b2c3d4e5f6g7h8i9j0klmnop.dsql.us-east-1.on.aws",
    CustomCredentialsProvider: credsProvider,
})
```

### Pool Configuration Tuning

Pool settings are configured via an optional `*pgxpool.Config` parameter to `NewPool`.
When omitted, the connector applies DSQL-specific defaults:
- `MaxConnLifetime`: 55 minutes (connections timeout after 60 minutes)
- `MaxConnIdleTime`: 10 minutes

All other fields use pgxpool defaults. To customize, create a config via
`pgxpool.ParseConfig` and override the fields you need:

```go
poolCfg, _ := pgxpool.ParseConfig("")
poolCfg.MaxConns = 20
poolCfg.MinConns = 5
poolCfg.MaxConnLifetime = time.Hour
poolCfg.MaxConnIdleTime = 30 * time.Minute
poolCfg.MaxConnLifetimeJitter = 5 * time.Minute
poolCfg.HealthCheckPeriod = time.Minute

pool, err := dsql.NewPool(ctx, dsql.Config{
    Host: "a1b2c3d4e5f6g7h8i9j0klmnop.dsql.us-east-1.on.aws",
}, poolCfg)
```

Setting `MaxConnLifetimeJitter` is recommended to prevent all connections from expiring simultaneously, which can cause a thundering herd of reconnections.

See [pgxpool.Config](https://pkg.go.dev/github.com/jackc/pgx/v5/pgxpool#Config) for all available options.

### Single Connection Usage

For simple scripts or when connection pooling is not needed:

```go
conn, err := dsql.Connect(ctx, dsql.Config{
    Host: "a1b2c3d4e5f6g7h8i9j0klmnop.dsql.us-east-1.on.aws",
})
if err != nil {
    log.Fatal(err)
}
defer conn.Close(ctx)

// Use the connection
rows, err := conn.Query(ctx, "SELECT * FROM users")
```

### Using AWS Profiles

Specify an AWS profile for credentials:

```go
pool, err := dsql.NewPool(ctx, dsql.Config{
    Host:    "a1b2c3d4e5f6g7h8i9j0klmnop.dsql.us-east-1.on.aws",
    Profile: "production",
})
```

## Token Generation

The connector automatically generates IAM authentication tokens:

- **Connection pools**: The `BeforeConnect` hook generates a fresh token for each new connection. Token generation is a local SigV4 presigning operation (no network calls), so this adds negligible overhead.
- **Single connections**: A fresh token is generated at connection time.
- **Credentials resolution**: AWS credentials are resolved once when the pool/connection is created and reused for all token generations, avoiding repeated credential chain resolution.

For the `admin` user, the connector generates admin tokens using `GenerateDBConnectAdminAuthToken`. For other users, it generates standard tokens using `GenerateDbConnectAuthToken`.

Token duration defaults to 15 minutes (recommended). The maximum allowed token lifetime is 1 week.

## OCC Retry

Aurora DSQL uses optimistic concurrency control (OCC). When two transactions
modify the same data, the first to commit wins and the second receives an OCC
error (codes `OC000`, `OC001`, or `40001`). The `occretry` package provides
two ways to handle these conflicts.

### Installation

```bash
go get github.com/awslabs/aurora-dsql-connectors/go/pgx/occretry
```

### Option 1: DB Interface (Pool-Level Retry)

Wrap your pool with `occretry.New` to get a `DB` that automatically retries
`Exec` and `Query` calls on OCC conflicts. On OCC conflict the entire
operation is re-executed, so callbacks passed to `WithTransaction` should
contain only database operations and be safe to retry.

```go
pool, _ := dsql.NewPool(ctx, dsql.Config{
    Host: "a1b2c3d4e5f6g7h8i9j0klmnop.dsql.us-east-1.on.aws",
})
defer pool.Close()

// Opt in to automatic OCC retry
db := occretry.New(pool, occretry.DefaultConfig())

// DDL — automatically retried on OCC conflict
db.Exec(ctx, "CREATE TABLE users (id UUID PRIMARY KEY DEFAULT gen_random_uuid(), name TEXT)")

// Transactions — entire transaction retried on OCC conflict
db.WithTransaction(ctx, func(tx pgx.Tx) error {
    _, err := tx.Exec(ctx, "UPDATE accounts SET balance = balance - $1 WHERE id = $2", amount, fromID)
    if err != nil {
        return err
    }
    _, err = tx.Exec(ctx, "UPDATE accounts SET balance = balance + $1 WHERE id = $2", amount, toID)
    return err
})

// Opt out for a single call
db.Exec(occretry.NoRetry(ctx), "SELECT 1")

// Or use the pool directly for operations that don't need retry
pool.QueryRow(ctx, "SELECT balance FROM accounts WHERE id = $1", id).Scan(&balance)
```

### Option 2: Helper Functions (Per-Call Retry)

Use the standalone helpers for explicit per-call retry control:

```go
// Retry any operation
err := occretry.Retry(ctx, occretry.DefaultConfig(), func() error {
    _, err := pool.Exec(ctx, "CREATE TABLE t (id UUID PRIMARY KEY)")
    return err
})

// Retry a single SQL statement
err = occretry.ExecWithRetry(ctx, pool, occretry.DefaultConfig(),
    "CREATE INDEX ASYNC ON users (email)")

// Retry a transaction
err = occretry.WithRetry(ctx, pool, occretry.DefaultConfig(), func(tx pgx.Tx) error {
    _, err := tx.Exec(ctx, "UPDATE accounts SET balance = balance - $1 WHERE id = $2", amount, fromID)
    if err != nil {
        return err
    }
    _, err = tx.Exec(ctx, "UPDATE accounts SET balance = balance + $1 WHERE id = $2", amount, toID)
    return err
})
```

### Custom Retry Configuration

```go
cfg := occretry.DefaultConfig()
cfg.MaxRetries = 5                        // default: 3
cfg.InitialWait = 200 * time.Millisecond  // default: 100ms
cfg.MaxWait = 10 * time.Second            // default: 5s

db := occretry.New(pool, cfg)
```

### Detecting OCC Errors

```go
if occretry.IsOCCError(err) {
    // Handle OCC conflict manually if needed
}
```

## Development

### Build

```bash
cd go/pgx
go build ./...
```

### Run Tests

Unit tests (no cluster required):

```bash
go test ./dsql/...
```

Integration tests (requires a DSQL cluster):

```bash
export CLUSTER_ENDPOINT="a1b2c3d4e5f6g7h8i9j0klmnop.dsql.us-east-1.on.aws"
go test ./example/test/...
```

### Lint

```bash
golangci-lint run
```

## Examples

The `example/` directory contains runnable examples demonstrating various patterns:

| Example | Description |
|---------|-------------|
| [example_preferred](example/src/example_preferred.go) | Recommended: Connection pool with concurrent queries |
| [transaction](example/src/transaction/) | Transaction handling with BEGIN/COMMIT/ROLLBACK |
| [occ_retry](example/src/occ_retry/) | Handling OCC conflicts with helper functions and DB interface |
| [connection_string](example/src/connection_string/) | Using connection strings for configuration |
| [manual_token](example/src/alternatives/manual_token/) | Manual IAM token generation without the connector |

### Running examples

```bash
export CLUSTER_ENDPOINT=a1b2c3d4e5f6g7h8i9j0klmnop.dsql.us-east-1.on.aws
cd example

# Run the preferred example
go run ./src/example_preferred.go

# Run the transaction example
go run ./src/transaction/...

# Run the OCC retry example
go run ./src/occ_retry/...

# Run the connection string example
go run ./src/connection_string/...
```

## DSQL Best Practices

When using this connector with Aurora DSQL, follow these practices:

1. **UUID Primary Keys**: Always use `UUID DEFAULT gen_random_uuid()` - DSQL doesn't support sequences or SERIAL
2. **OCC Handling**: DSQL uses optimistic concurrency control. Handle error codes `OC000` (data conflict) and `OC001` (schema conflict) with retry logic
3. **No Foreign Keys**: DSQL doesn't support foreign key constraints - enforce relationships in your application
4. **Async Indexes**: Use `CREATE INDEX ASYNC` for index creation
5. **Transaction Limits**: Transactions are limited to 3,000 rows, 10 MiB, and 5 minutes
6. **Connection Limits**: Connections timeout after 60 minutes; configure `MaxConnLifetime` on your `pgxpool.Config` accordingly
7. **No SAVEPOINT**: Partial rollbacks via SAVEPOINT are not supported

## Additional Resources

- [Amazon Aurora DSQL Documentation](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/what-is-aurora-dsql.html)
- [pgx Documentation](https://pkg.go.dev/github.com/jackc/pgx/v5)
- [pgxpool Documentation](https://pkg.go.dev/github.com/jackc/pgx/v5/pgxpool)
- [AWS SDK for Go v2](https://aws.github.io/aws-sdk-go-v2/)

---

Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.

SPDX-License-Identifier: Apache-2.0
