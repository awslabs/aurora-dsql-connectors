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
- OCC retry utilities for handling optimistic concurrency conflicts

## Prerequisites

- Ruby 3.1 or later
- AWS credentials configured
- An Aurora DSQL cluster

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

# Execute queries
pool.with do |conn|
  result = conn.exec("SELECT 'Hello, DSQL!'")
  puts result[0]["?column?"]
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

The `Connection` wrapper delegates common methods (`exec`, `exec_params`, `query`, `transaction`, `close`, `finished?`) directly. All other `PG::Connection` methods (e.g., `prepare`, `exec_prepared`, `copy_data`) are also available via delegation. The underlying `PG::Connection` can be accessed directly via `conn.pg_conn` if needed.

## OCC Retry

Aurora DSQL uses optimistic concurrency control. Handle conflicts with retry:

```ruby
AuroraDsql::Pg::OCCRetry.with_retry(pool) do |conn|
  conn.exec_params("UPDATE accounts SET balance = balance - $1 WHERE id = $2", [100, from_id])
  conn.exec_params("UPDATE accounts SET balance = balance + $1 WHERE id = $2", [100, to_id])
end
```

## Development

```bash
cd ruby/pg
bundle install
bundle exec rake unit        # Run unit tests
bundle exec rake integration # Run integration tests (requires CLUSTER_ENDPOINT)
```

## DSQL Best Practices

- Use `UUID DEFAULT gen_random_uuid()` for primary keys (no sequences)
- Handle OCC errors (OC000, OC001) with retry logic
- Use `CREATE INDEX ASYNC` for index creation
- No foreign keys, triggers, or temp tables
- Transaction limits: 3,000 rows, 10 MiB, 5 minutes

## License

Apache-2.0
