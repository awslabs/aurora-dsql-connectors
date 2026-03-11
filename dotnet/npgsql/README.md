# Aurora DSQL Npgsql Connector for .NET

## Overview

A .NET connector for Amazon Aurora DSQL that wraps [Npgsql](https://www.npgsql.org/) with automatic IAM authentication. The connector handles token generation, SSL configuration, and connection pooling so you can focus on your application logic.

## Features

- Automatic IAM token generation (admin and regular users)
- Connection pooling via `NpgsqlDataSource` with max lifetime enforcement
- Single connection support for simpler use cases
- Flexible host configuration (full endpoint or cluster ID)
- Region auto-detection from endpoint hostname
- Support for AWS profiles and custom credentials providers
- SSL always enabled with `verify-full` mode and direct TLS negotiation
- Opt-in OCC retry with exponential backoff and jitter
- Connection string parsing support

## Prerequisites

- .NET 8.0 or later
- AWS credentials configured (see [Credentials Resolution](#credentials-resolution) below)
- An Aurora DSQL cluster

For information about creating an Aurora DSQL cluster, see the [Getting started with Aurora DSQL](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/getting-started.html) guide.

### Credentials Resolution

The connector uses the [AWS SDK for .NET default credential chain](https://docs.aws.amazon.com/sdk-for-net/v3/developer-guide/creds-assign.html), which resolves credentials in the following order:

1. **Environment variables** (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and optionally `AWS_SESSION_TOKEN`)
2. **Shared credentials file** (`~/.aws/credentials`) with optional profile via `AWS_PROFILE` or `DsqlConfig.Profile`
3. **Shared config file** (`~/.aws/config`)
4. **IAM role for Amazon EC2/ECS/Lambda** (instance metadata or task role)

The first source that provides valid credentials is used. You can override this by specifying `Profile` for a specific AWS profile or `CustomCredentialsProvider` for complete control over credential resolution.

## Installation

```bash
dotnet add package Amazon.AuroraDsql.Npgsql
```

## Quick Start

```csharp
using Amazon.AuroraDsql.Npgsql;

// Create a connection pool (recommended)
await using var ds = AuroraDsql.CreateDataSource(new DsqlConfig
{
    Host = "your-cluster.dsql.us-east-1.on.aws"
});

// Read
await using (var conn = await ds.OpenConnectionAsync())
{
    await using var cmd = conn.CreateCommand();
    cmd.CommandText = "SELECT 'Hello, DSQL!'";
    var greeting = await cmd.ExecuteScalarAsync();
    Console.WriteLine(greeting);
}

// Transactional write
await OccRetry.WithTransactionRetryAsync(ds, maxRetries: 3, async conn =>
{
    await using var cmd = conn.CreateCommand();
    cmd.CommandText = "INSERT INTO users (id, name) VALUES (gen_random_uuid(), @name)";
    cmd.Parameters.AddWithValue("name", "Alice");
    await cmd.ExecuteNonQueryAsync();
});
```

## Configuration Options

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `Host` | `string` | (required) | Cluster endpoint or 26-char cluster ID |
| `Region` | `string?` | (auto-detected) | AWS region; required if Host is a cluster ID |
| `User` | `string` | `"admin"` | Database user |
| `Database` | `string` | `"postgres"` | Database name |
| `Port` | `int` | `5432` | Database port |
| `Profile` | `string?` | `null` | AWS profile name for credentials |
| `CustomCredentialsProvider` | `AWSCredentials?` | `null` | Custom AWS credentials provider |
| `MaxPoolSize` | `int` | `10` | Maximum pool connections |
| `MinPoolSize` | `int` | `0` | Minimum pool connections |
| `ConnectionLifetime` | `int` | `3300` (55 min) | Max connection lifetime in seconds |
| `ConnectionIdleLifetime` | `int` | `600` (10 min) | Max idle time before connection is closed |
| `OccMaxRetries` | `int?` | `null` (disabled) | Max OCC retries on `ExecuteAsync`; enables retry when set |
| `OrmPrefix` | `string?` | `null` | ORM prefix prepended to `application_name` (e.g., `"efcore"`) |
| `LoggerFactory` | `ILoggerFactory?` | `null` | Logger factory for retry warnings and diagnostics |
| `ConfigureConnectionString` | `Action<NpgsqlConnectionStringBuilder>?` | `null` | Callback to set additional Npgsql connection string properties after defaults |

## Connection String Format

The connector supports `postgres://` and `postgresql://` connection string formats:

```
postgres://[user@]host[:port]/[database][?param=value&...]
postgresql://[user@]host[:port]/[database][?param=value&...]
```

**Supported query parameters:**
- `region` - AWS region
- `profile` - AWS profile name

**Examples:**

```csharp
// Full endpoint (region auto-detected)
await using var ds = AuroraDsql.CreateDataSource(
    "postgres://admin@cluster.dsql.us-east-1.on.aws/postgres");

// With AWS profile
await using var ds = AuroraDsql.CreateDataSource(
    "postgres://admin@cluster.dsql.us-east-1.on.aws/postgres?profile=dev");
```

## Single Connection Usage

For simple scripts or when connection pooling is not needed:

```csharp
await using var conn = await AuroraDsql.ConnectAsync(new DsqlConfig
{
    Host = "your-cluster.dsql.us-east-1.on.aws"
});

await using var cmd = conn.CreateCommand("SELECT * FROM users");
await using var reader = await cmd.ExecuteReaderAsync();
while (await reader.ReadAsync())
{
    Console.WriteLine(reader.GetString(0));
}
```

You can also connect from a connection string:

```csharp
await using var conn = await AuroraDsql.ConnectAsync(
    "postgres://admin@cluster.dsql.us-east-1.on.aws/postgres");
```

The `DsqlConnection` wrapper delegates `CreateCommand` and exposes the underlying `NpgsqlConnection` via `conn.Connection` for advanced use.

## OCC Retry

Aurora DSQL uses optimistic concurrency control (OCC). When two transactions modify the same data, the first to commit wins and the second receives an OCC error (SQLSTATE `40001`).

### Transaction retry with `WithTransactionRetryAsync`

Manages `BEGIN`/`COMMIT`/`ROLLBACK` internally via raw SQL (DSQL uses fixed Repeatable Read isolation, so the isolation level clause that Npgsql's `BeginTransactionAsync` sends is unnecessary). Opens a fresh connection for each attempt:

```csharp
await OccRetry.WithTransactionRetryAsync(ds, maxRetries: 3, async conn =>
{
    await using var cmd = conn.CreateCommand();

    cmd.CommandText = "UPDATE accounts SET balance = balance - 100 WHERE id = @from";
    cmd.Parameters.AddWithValue("from", fromId);
    await cmd.ExecuteNonQueryAsync();

    cmd.CommandText = "UPDATE accounts SET balance = balance + 100 WHERE id = @to";
    cmd.Parameters.Clear();
    cmd.Parameters.AddWithValue("to", toId);
    await cmd.ExecuteNonQueryAsync();
});
```

### Single SQL retry with `ExecWithRetryAsync`

For DDL or single DML statements:

```csharp
await OccRetry.ExecWithRetryAsync(ds, "CREATE INDEX ASYNC idx_users_name ON users (name)", maxRetries: 3);
```

### Pool-level retry with `ExecuteAsync`

Use `ExecuteAsync` on the data source for automatic retry. Enable globally via `OccMaxRetries` in config, or per-call via `maxOccRetries`:

```csharp
// Global: set OccMaxRetries in config
var ds = AuroraDsql.CreateDataSource(new DsqlConfig
{
    Host = "your-cluster.dsql.us-east-1.on.aws",
    OccMaxRetries = 3
});

// Per-call: override with maxOccRetries parameter
await ds.ExecuteAsync(async conn =>
{
    await using var cmd = conn.CreateCommand();
    cmd.CommandText = "INSERT INTO users (id, name) VALUES (gen_random_uuid(), @name)";
    cmd.Parameters.AddWithValue("name", "Alice");
    await cmd.ExecuteNonQueryAsync();
}, maxOccRetries: 3);
```

> **Note:** `ExecuteAsync` does NOT wrap the action in a transaction. For transactional
> writes, use `OccRetry.WithTransactionRetryAsync` which manages `BEGIN`/`COMMIT`/`ROLLBACK`
> automatically.

### OCC error detection

To detect OCC errors in custom retry logic:

```csharp
try
{
    // ... database operations
}
catch (Exception ex) when (OccRetry.IsOccError(ex))
{
    // Handle OCC conflict
}
```

## Examples

The `example/` directory contains runnable examples demonstrating various patterns:

| Example | Description |
|---------|-------------|
| [ExamplePreferred](example/src/ExamplePreferred.cs) | Recommended: Connection pool with concurrent queries and transactional write |
| [SingleConnection](example/src/alternatives/SingleConnection/) | Single connection without pooling |
| [ManualToken](example/src/alternatives/ManualToken/) | Manual IAM token generation without the connector |

### Running examples

```bash
export CLUSTER_ENDPOINT=your-cluster.dsql.us-east-1.on.aws
cd example

dotnet test --filter "ExamplePreferredTest"
dotnet test --filter "SingleConnectionExampleTest"
dotnet test --filter "ManualTokenExampleTest"
```

## Development

### Build

```bash
cd dotnet/npgsql
dotnet build src/Amazon.AuroraDsql.Npgsql/
```

### Run Tests

Unit tests (no cluster required):

```bash
dotnet test test/unit/Amazon.AuroraDsql.Npgsql.Tests/
```

Integration tests (requires a DSQL cluster):

```bash
export CLUSTER_ENDPOINT="your-cluster.dsql.us-east-1.on.aws"
dotnet test test/integration/Amazon.AuroraDsql.Npgsql.IntegrationTests/
```

### Format

```bash
dotnet format src/Amazon.AuroraDsql.Npgsql/
```

## DSQL Best Practices

When using this connector with Aurora DSQL, follow these practices:

1. **UUID Primary Keys**: Always use `UUID DEFAULT gen_random_uuid()` — DSQL doesn't support sequences or SERIAL
2. **OCC Handling**: DSQL uses optimistic concurrency control. Enable retry via `OccMaxRetries` in config; for single connections, use `OccRetry` explicitly
3. **No Foreign Keys**: Enforce referential integrity in your application
4. **Async Indexes**: Use `CREATE INDEX ASYNC` for index creation (max 24 indexes per table, max 8 columns per index)
5. **One DDL per Transaction**: Separate DDL and DML into distinct transactions
6. **Transaction Limits**: 3,000 rows, 10 MiB, and 5 minutes per transaction
7. **Connection Limits**: Connections timeout after 60 minutes; configure `ConnectionLifetime` accordingly (default 55 minutes). Max 10,000 connections per cluster
8. **Fixed Isolation Level**: DSQL uses Repeatable Read isolation — it cannot be changed
9. **No TRUNCATE**: Use `DELETE FROM table` instead
10. **No SAVEPOINT**: Partial rollbacks are not supported
11. **No Triggers**: Implement in your application layer
12. **No Temp Tables**: Use regular tables or application-level caching
13. **No Partitioning**: Manage data distribution in your application
14. **No Stored Procedures**: DSQL does not support `CALL` or PL/pgSQL
15. **No Extensions**: PL/pgSQL, PostGIS, pgvector, etc. are not available
16. **No PREPARE TRANSACTION**: Distributed transactions via `TransactionScope` / `System.Transactions` are disabled (`Enlist=false` is forced)
17. **Single Database**: DSQL always uses `postgres`
18. **Token Expiry**: IAM auth tokens are valid for 15 minutes (DSQL enforced maximum). The connector generates a fresh token for each new connection, so this is handled automatically
19. **Limited Type System**: Use VARCHAR, TEXT, INTEGER, DECIMAL, BOOLEAN, TIMESTAMP, UUID. Arrays and JSON types are not natively supported — store as TEXT and parse in your application
20. **No NativeAOT**: Npgsql source generators have not been tested with this connector due to AWS SDK dependencies
21. **Npgsql 10.x**: Root CA validation changes in Npgsql 10.x may require providing an explicit certificate path; this connector currently targets Npgsql 9.x

## Horizontal Scaling

When scaling your application horizontally with Aurora DSQL:

- **Pool sizing**: Configure `MaxPoolSize` between 10–50 connections per application instance. DSQL supports up to 10,000 concurrent connections per cluster, so keep per-instance pools modest.
- **Batch size**: Keep write batches between 500–1,000 rows per transaction to stay within DSQL's transaction limits (3,000 rows, 10 MiB).
- **UUID primary keys**: Always use `gen_random_uuid()` to avoid key collisions across instances.
- **Hot key avoidance**: Compute aggregates via `SELECT` queries instead of maintaining running counters. See [Avoiding Hot Keys](https://marc-bowes.com/dsql-avoid-hot-keys.html).
- **Retry on OCC conflicts**: With more instances, OCC conflicts become more likely on contended rows. Enable retry logic (`OccMaxRetries`) for write workloads.
- **Retry on internal errors**: Internal errors are retryable. The retry uses a new connection from the pool with backoff and jitter to avoid thundering herd.
- **Connection lifetime**: Keep `ConnectionLifetime` under 60 minutes (the default 55 minutes is recommended) to avoid server-side timeouts.
- **Fresh tokens per connection**: The connector generates a fresh IAM token for each new physical connection. Token generation is a local SigV4 presigning operation (no network calls), so this adds negligible overhead even at scale.

## Additional Resources

- [Amazon Aurora DSQL Documentation](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/what-is-aurora-dsql.html)
- [Npgsql Documentation](https://www.npgsql.org/doc/)
- [NpgsqlDataSource API Reference](https://www.npgsql.org/doc/api/Npgsql.NpgsqlDataSource.html)
- [AWS SDK for .NET](https://docs.aws.amazon.com/sdk-for-net/v3/developer-guide/welcome.html)

---

Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.

SPDX-License-Identifier: Apache-2.0
