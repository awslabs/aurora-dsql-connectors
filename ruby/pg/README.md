# Aurora DSQL Ruby pg Connector

## Overview

A Ruby connector for Amazon Aurora DSQL that wraps the [pg](https://github.com/ged/ruby-pg) gem with automatic IAM authentication. The connector handles token generation, SSL configuration, and connection pooling so you can focus on your application logic.

## Features

- Automatic IAM token generation with smart caching (refreshes at 80% of token lifetime)
- Connection pooling via `connection_pool` gem with max_lifetime enforcement
- Single connection support for simpler use cases
- Flexible host configuration (full endpoint or cluster ID)
- Region auto-detection from endpoint hostname
- Support for AWS profiles and custom credentials providers
- SSL always enabled with `verify-full` mode and direct TLS negotiation (libpq 17+)
- Automatic OCC retry with exponential backoff on `pool.with`

## Prerequisites

- Ruby 3.1 or later
- AWS credentials configured (see [Credentials Resolution](#credentials-resolution) below)
- An Aurora DSQL cluster

For information about creating an Aurora DSQL cluster, see the [Getting started with Aurora DSQL](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/getting-started.html) guide.

### Credentials Resolution

The connector uses the [AWS SDK for Ruby default credential chain](https://docs.aws.amazon.com/sdkref/latest/guide/standardized-credentials.html), which resolves credentials in the following order:

1. **Environment variables** (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and optionally `AWS_SESSION_TOKEN`)
2. **Shared credentials file** (`~/.aws/credentials`) with optional profile via `AWS_PROFILE` or `profile` config
3. **Shared config file** (`~/.aws/config`)
4. **IAM role for Amazon EC2/ECS/Lambda** (instance metadata or task role)

The first source that provides valid credentials is used. You can override this by specifying `profile` for a specific AWS profile or `credentials_provider` for complete control over credential resolution.

## Installation

Add to your Gemfile:

```ruby
gem "aurora-dsql-ruby-pg"
```

Or install directly:

```bash
gem install aurora-dsql-ruby-pg
```

## Quick Start

```ruby
require "aurora_dsql_pg"

# Create a connection pool
pool = AuroraDsql::Pg.create_pool(
  host: "your-cluster.dsql.us-east-1.on.aws"
)

# Read — no OCC retry needed for reads
pool.with(retry_occ: false) do |conn|
  result = conn.exec("SELECT 'Hello, DSQL!'")
  puts result[0]["?column?"]
end

# Write — OCC retry is automatic, but you must wrap writes in a transaction
pool.with do |conn|
  conn.transaction do
    conn.exec_params("INSERT INTO users (id, name) VALUES (gen_random_uuid(), $1)", ["Alice"])
  end
end

pool.shutdown
```

## Configuration Options

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `host` | `String` | (required) | Cluster endpoint or cluster ID |
| `region` | `String` | (auto-detected) | AWS region |
| `user` | `String` | `"admin"` | Database user |
| `database` | `String` | `"postgres"` | Database name |
| `port` | `Integer` | `5432` | Database port |
| `profile` | `String` | `nil` | AWS profile name |
| `token_duration` | `Integer` | `900` (15 min) | Token validity in seconds |
| `credentials_provider` | `Aws::Credentials` | `nil` | Custom credentials |
| `pool_size` | `Integer` | `5` | Connection pool size |
| `max_lifetime` | `Integer` | `3300` (55 min) | Max connection lifetime in seconds |
| `application_name` | `String` | `nil` | ORM prefix for application_name |
| `logger` | `Logger` | `nil` | Logger for OCC retry warnings |

## Connection String Format

```ruby
pool = AuroraDsql::Pg.create_pool(
  "postgres://admin@cluster.dsql.us-east-1.on.aws/postgres?profile=dev"
)
```

## Single Connection Usage

```ruby
conn = AuroraDsql::Pg.connect(host: "cluster.dsql.us-east-1.on.aws")
conn.exec("SELECT 1")
conn.close
```

The `Connection` wrapper delegates common methods (`exec_params`, `query`, `transaction`, `close`, `finished?`) directly. All other `PG::Connection` methods (e.g., `exec`, `prepare`, `exec_prepared`, `copy_data`) are also available via delegation. The underlying `PG::Connection` can be accessed directly via `conn.pg_conn` if needed.

## OCC Retry

Aurora DSQL uses optimistic concurrency control (OCC). When two transactions modify the same data, the first to commit wins and the second receives an OCC error.

`pool.with` automatically retries on OCC errors (up to 3 retries with exponential backoff and jitter). No special code is needed.

> **Important:** `pool.with` does NOT automatically wrap your block in a transaction. You must call `conn.transaction` yourself for write operations. On OCC conflict the entire block is re-executed, so it should contain only database operations and be safe to retry.

```ruby
pool.with do |conn|
  conn.transaction do
    conn.exec_params("UPDATE accounts SET balance = balance - $1 WHERE id = $2", [100, from_id])
    conn.exec_params("UPDATE accounts SET balance = balance + $1 WHERE id = $2", [100, to_id])
  end
end
```

To disable automatic retry (e.g., for read-only queries):

```ruby
pool.with(retry_occ: false) do |conn|
  conn.exec("SELECT 1")
end
```

For custom retry configuration (different max retries, backoff, etc.), use the `OCCRetry` module directly. Unlike `pool.with`, `OCCRetry.with_retry` automatically wraps the block in a transaction:

```ruby
AuroraDsql::Pg::OCCRetry.with_retry(pool, max_retries: 10) do |conn|
  conn.exec_params("UPDATE ...", [...])
end
```

For single SQL statements (DDL or DML), `exec_with_retry` provides a simple convenience without transaction wrapping:

```ruby
AuroraDsql::Pg::OCCRetry.exec_with_retry(pool, "CREATE TABLE users (id UUID PRIMARY KEY)")
```

To see OCC retries in your logs, pass a `logger` when creating the pool:

```ruby
pool = AuroraDsql::Pg.create_pool(
  host: "your-cluster.dsql.us-east-1.on.aws",
  logger: Logger.new(STDOUT)
)
```

## Examples

The `example/` directory contains runnable examples demonstrating various patterns:

| Example | Description |
|---------|-------------|
| [example_preferred](example/src/example_preferred.rb) | Recommended: Connection pool with concurrent queries |
| [manual_token](example/src/alternatives/manual_token/) | Manual IAM token generation without the connector |

### Running examples

```bash
export CLUSTER_ENDPOINT=your-cluster.dsql.us-east-1.on.aws
export CLUSTER_USER=admin
export REGION=us-east-1
cd example

# Run the preferred example
ruby src/example_preferred.rb

# Run the manual token example
ruby src/alternatives/manual_token/example.rb
```

## Development

```bash
cd ruby/pg
bundle install
bundle exec rake unit        # Run unit tests
bundle exec rake integration # Run integration tests (requires CLUSTER_ENDPOINT)
```

## DSQL Best Practices

When using this connector with Aurora DSQL, follow these practices:

1. **UUID Primary Keys**: Always use `UUID DEFAULT gen_random_uuid()` - DSQL doesn't support sequences or SERIAL
2. **OCC Handling**: DSQL uses optimistic concurrency control. `pool.with` retries OCC errors automatically; for single connections, use `OCCRetry` explicitly
3. **No Foreign Keys**: DSQL doesn't support foreign key constraints - enforce relationships in your application
4. **Async Indexes**: Use `CREATE INDEX ASYNC` for index creation
5. **Transaction Limits**: Transactions are limited to 3,000 rows, 10 MiB, and 5 minutes
6. **Connection Limits**: Connections timeout after 60 minutes; configure pool `max_lifetime` accordingly
7. **No SAVEPOINT**: Partial rollbacks via SAVEPOINT are not supported

## Additional Resources

- [Amazon Aurora DSQL Documentation](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/what-is-aurora-dsql.html)
- [pg gem Documentation](https://deveiate.org/code/pg/)
- [connection_pool Documentation](https://github.com/mperham/connection_pool)
- [AWS SDK for Ruby](https://docs.aws.amazon.com/sdk-for-ruby/v3/api/)

---

Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.

SPDX-License-Identifier: Apache-2.0
