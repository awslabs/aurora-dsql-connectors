# Aurora DSQL PHP PDO_PGSQL Connector

## Overview

A PHP connector for Amazon Aurora DSQL that wraps [PDO_PGSQL](https://www.php.net/manual/en/ref.pdo-pgsql.php) with automatic IAM authentication, SSL enforcement (`sslmode=verify-full`), and built-in OCC retry with exponential backoff. The connector handles token generation and connection configuration so you can focus on your application logic.

## Features

- Automatic IAM token generation via AWS SDK for PHP
- SSL always enabled with `verify-full` mode and direct TLS negotiation (libpq 17+)
- Flexible host configuration (full endpoint or cluster ID)
- Region auto-detection from endpoint hostname
- Support for AWS profiles and custom credentials providers
- OCC retry with exponential backoff and jitter
- PSR-3 compatible logging for retry diagnostics
- Connection string (`postgres://`) parsing support
- PDO attribute overrides for full control over connection behavior

## Prerequisites

- PHP 8.2 or later
- `ext-pdo_pgsql` extension
- AWS credentials configured (see [Credentials Resolution](#credentials-resolution) below)
- An Aurora DSQL cluster

For information about creating an Aurora DSQL cluster, see the [Getting started with Aurora DSQL](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/getting-started.html) guide.

### Credentials Resolution

The connector uses the [AWS SDK for PHP default credential chain](https://docs.aws.amazon.com/sdkref/latest/guide/standardized-credentials.html), which resolves credentials in the following order:

1. **Environment variables** (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and optionally `AWS_SESSION_TOKEN`)
2. **Shared credentials file** (`~/.aws/credentials`) with optional profile via `AWS_PROFILE` or `profile` config
3. **Shared config file** (`~/.aws/config`)
4. **IAM role for Amazon EC2/ECS/Lambda** (instance metadata or task role)

The first source that provides valid credentials is used. You can override this by specifying `profile` for a specific AWS profile or `credentialsProvider` for complete control over credential resolution.

## Installation

```bash
composer require awslabs/aurora-dsql-pdo-pgsql
```

## Quick Start

```php
<?php

require_once 'vendor/autoload.php';

use Aws\AuroraDsql\PdoPgsql\AuroraDsql;
use Aws\AuroraDsql\PdoPgsql\DsqlConfig;

$config = new DsqlConfig(
    host: 'your-cluster.dsql.us-east-1.on.aws',
    occMaxRetries: 3,
);
$pdo = AuroraDsql::connect($config);

// Simple read — $pdo is a \PDO, use it normally
$stmt = $pdo->query('SELECT 1 AS result');
$row = $stmt->fetch(PDO::FETCH_ASSOC);
echo "Connected: {$row['result']}\n";

// Transactional write with automatic OCC retry
$id = $pdo->transaction(function (PDO $conn): string {
    $stmt = $conn->prepare('INSERT INTO users (name) VALUES (?) RETURNING id');
    $stmt->execute(['Alice']);
    return $stmt->fetchColumn();
});
```

## Configuration Options

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `host` | `string` | (required) | Cluster endpoint or cluster ID |
| `region` | `?string` | `null` (auto-detected) | AWS region |
| `user` | `string` | `'admin'` | Database user |
| `database` | `string` | `'postgres'` | Database name |
| `port` | `int` | `5432` | Database port |
| `profile` | `?string` | `null` | AWS profile name |
| `credentialsProvider` | `?\Closure` | `null` | Custom credentials provider |
| `tokenDurationSecs` | `int` | `900` (15 min) | Token validity in seconds |
| `ormPrefix` | `?string` | `null` | ORM prefix for `application_name` |
| `occMaxRetries` | `?int` | `null` (disabled) | Max OCC retries for `exec()` and `transaction()` |
| `logger` | `?LoggerInterface` | `null` | PSR-3 logger for retry diagnostics |

Both `connect()` and `connectFromDsn()` accept a `pdoAttributes` array for PDO attribute overrides:

```php
$pdo = AuroraDsql::connect($config, pdoAttributes: [
    PDO::ATTR_DEFAULT_FETCH_MODE => PDO::FETCH_ASSOC,
]);
```

> **Note on persistent connections:** `PDO::ATTR_PERSISTENT` won't provide connection reuse with DSQL. The connector generates a fresh IAM token per connection, changing the effective connection string each time, which prevents PDO's connection pool from matching existing connections. It's not harmful, but it won't improve performance.

> **Note on connection lifetime:** Aurora DSQL enforces a 60-minute maximum connection lifetime. For long-lived processes (CLI workers, daemons, queue consumers), you must reconnect before the limit is reached. A safe pattern is to recreate the PDO instance every 55 minutes or catch the resulting `PDOException` and reconnect.

## Host Configuration

The connector supports two host formats:

**Full endpoint** (region auto-detected from hostname):
```php
$config = new DsqlConfig(host: 'a1b2c3d4e5f6g7h8i9j0klmnop.dsql.us-east-1.on.aws');
```

**Cluster ID** (26-character ID; region required):
```php
$config = new DsqlConfig(
    host: 'a1b2c3d4e5f6g7h8i9j0klmnop',
    region: 'us-east-1',
);
```

When using a cluster ID, the region can also be set via `AWS_REGION` or `AWS_DEFAULT_REGION` environment variables.

## Token Generation

The connector automatically generates IAM authentication tokens when connecting:

- For the `admin` user, the connector uses `generateDbConnectAdminAuthToken`
- For all other users, it uses `generateDbConnectAuthToken`

Token generation is a local SigV4 presigning operation (no network calls), so it adds negligible overhead. Token duration defaults to 900 seconds (15 minutes).

## Connection String Format

The connector supports `postgres://` and `postgresql://` connection strings with DSQL-specific query parameters:

```
postgres://admin@cluster.dsql.us-east-1.on.aws/postgres?region=us-east-1&profile=dev&tokenDurationSecs=900
```

```php
$pdo = AuroraDsql::connectFromDsn(
    'postgres://admin@your-cluster.dsql.us-east-1.on.aws/postgres'
);
```

## OCC Retry

Aurora DSQL uses optimistic concurrency control (OCC). When two transactions modify the same data, the first to commit wins and the second receives an OCC error (SQLSTATE `40001`, `OC000`, or `OC001`).

Enable OCC retry once at connection time via `occMaxRetries`, and retries apply automatically to `exec()` and `transaction()`:

```php
$config = new DsqlConfig(
    host: 'your-cluster.dsql.us-east-1.on.aws',
    occMaxRetries: 3,  // enable OCC retry
);
$pdo = AuroraDsql::connect($config);

// Single statements are automatically retried via exec()
$pdo->exec("CREATE INDEX ASYNC ON users (email)");

// Multi-statement transactions are retried via transaction()
$pdo->transaction(function (PDO $conn) {
    $conn->exec("UPDATE accounts SET balance = balance - 100 WHERE id = 1");
    $conn->exec("UPDATE accounts SET balance = balance + 100 WHERE id = 2");
});
```

> **Idempotency:** When OCC retry is enabled, operations may be re-executed. Ensure that retried statements and transaction callbacks are **idempotent** — safe to run more than once. For single `exec()` calls, this means the statement itself must be safe to repeat. For `transaction()`, the entire callback is re-executed on retry.

### `$pdo->exec()`

When `occMaxRetries` is set, `exec()` automatically retries single statements on OCC conflict with exponential backoff. No explicit transaction wrapping is applied, making this suitable for both DDL (`CREATE TABLE`, `CREATE INDEX ASYNC`) and single DML statements.

When called inside a transaction (via `transaction()` or manual `beginTransaction()`), `exec()` delegates to the parent without retry — retries are handled at the transaction level.

### `$pdo->transaction()`

The `transaction()` method handles `beginTransaction()`/`commit()`/`rollBack()` automatically and retries on OCC conflict with exponential backoff (100ms initial, 2x multiplier, 5s max) with jitter.

> **Important:** Do NOT call `beginTransaction()` or `commit()` inside the callback — `transaction()` manages the transaction lifecycle. On OCC conflict the entire callback is re-executed, so it must be safe to retry.

You can override the retry count per-call:

```php
$pdo->transaction(function (PDO $conn) {
    // ...
}, maxRetries: 5);
```

For parameterized writes with `prepare()`/`execute()`, use `transaction()` to get automatic retry:

```php
$pdo->transaction(function (PDO $conn) use ($name) {
    $stmt = $conn->prepare('INSERT INTO users (name) VALUES (?)');
    $stmt->execute([$name]);
});
```

### Disabling retry per-call

To disable retry for a specific `transaction()` call, pass `maxRetries: 0`. `exec()` already skips retry when called inside a transaction — no special handling needed.

### `OCCRetry::isOccError()`

Checks whether a `\Throwable` is an OCC error. Useful for manual error detection.

```php
try {
    $pdo->exec('...');
} catch (\PDOException $e) {
    if (OCCRetry::isOccError($e)) {
        // handle OCC conflict
    }
}
```

### Logging

Pass a PSR-3 `LoggerInterface` via config to see retry diagnostics:

```php
$config = new DsqlConfig(
    host: 'your-cluster.dsql.us-east-1.on.aws',
    occMaxRetries: 3,
    logger: $logger,
);
```

## Examples

| Example | Description |
|---------|-------------|
| [example_preferred](example/src/example_preferred.php) | Recommended: connect and perform reads/writes with OCC retry |
| [manual_token](example/src/alternatives/manual_token/manual_token.php) | Manual IAM token generation without the connector |

### Running examples

```bash
cd example && composer install && cd ..
export CLUSTER_ENDPOINT=your-cluster.dsql.us-east-1.on.aws
php example/src/example_preferred.php
```

## Additional Resources

- [Aurora DSQL User Guide](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/what-is-aurora-dsql.html)
- [Aurora DSQL PostgreSQL Compatibility](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/working-with-postgresql-compatibility.html)
- [Aurora DSQL Unsupported Features](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/working-with-postgresql-compatibility-unsupported.html)
- [Optimistic Concurrency Control in Aurora DSQL](https://aws.amazon.com/blogs/database/introducing-optimistic-concurrency-control-in-amazon-aurora-dsql/)
- [PHP PDO Documentation](https://www.php.net/manual/en/book.pdo.php)
- [AWS SDK for PHP](https://docs.aws.amazon.com/sdk-for-php/v3/developer-guide/welcome.html)

---

Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.

SPDX-License-Identifier: Apache-2.0
