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
await using var ds = await AuroraDsql.CreateDataSourceAsync(new DsqlConfig
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

// Transactional write with OCC retry
await ds.WithTransactionRetryAsync(async conn =>
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
| `OccMaxRetries` | `int?` | `null` (disabled) | Default max OCC retries for retry methods on the data source |
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
await using var ds = await AuroraDsql.CreateDataSourceAsync(
    "postgres://admin@cluster.dsql.us-east-1.on.aws/postgres");

// With AWS profile
await using var ds = await AuroraDsql.CreateDataSourceAsync(
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

Use `WithTransactionRetryAsync` on the data source for transactional writes with automatic OCC retry. It manages `BEGIN`/`COMMIT`/`ROLLBACK` internally and opens a fresh connection for each attempt. Set `OccMaxRetries` in config for the default, or override per-call:

```csharp
await using var ds = await AuroraDsql.CreateDataSourceAsync(new DsqlConfig
{
    Host = "your-cluster.dsql.us-east-1.on.aws",
    OccMaxRetries = 3
});

await ds.WithTransactionRetryAsync(async conn =>
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

The static `OccRetry.WithTransactionRetryAsync` overloads are also available for use with a raw `NpgsqlDataSource`.

### Single SQL retry with `ExecWithRetryAsync`

For DDL or single DML statements:

```csharp
await ds.ExecWithRetryAsync("CREATE INDEX ASYNC idx_users_name ON users (name)");
```

The static `OccRetry.ExecWithRetryAsync` overloads are also available for use with a raw `NpgsqlDataSource`.

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

## Additional Resources

- [Amazon Aurora DSQL Documentation](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/what-is-aurora-dsql.html)
- [SQL Feature Compatibility in Aurora DSQL](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/working-with-postgresql-compatibility.html)
- [Aurora DSQL and PostgreSQL](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/working-with.html)
- [Npgsql Documentation](https://www.npgsql.org/doc/)
- [NpgsqlDataSource API Reference](https://www.npgsql.org/doc/api/Npgsql.NpgsqlDataSource.html)
- [AWS SDK for .NET](https://docs.aws.amazon.com/sdk-for-net/v3/developer-guide/welcome.html)

---

Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.

SPDX-License-Identifier: Apache-2.0
