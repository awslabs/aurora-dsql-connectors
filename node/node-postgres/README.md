# Aurora DSQL Connector for node-postgres

[![GitHub](https://img.shields.io/badge/github-awslabs/aurora--dsql--connectors-blue?logo=github)](https://github.com/awslabs/aurora-dsql-connectors/tree/main/node/node-postgres)
[![License](https://img.shields.io/badge/license-Apache--2.0-brightgreen)](https://github.com/awslabs/aurora-dsql-connectors/blob/main/LICENSE)
[![NPM Version](https://img.shields.io/npm/v/@aws/aurora-dsql-node-postgres-connector)](https://www.npmjs.com/package/@aws/aurora-dsql-node-postgres-connector)
[![Discord chat](https://img.shields.io/discord/1435027294837276802.svg?logo=discord)](https://discord.com/invite/nEF6ksFWru)

The Aurora DSQL Connector for node-postgres is a Node.js connector built on [node-postgres](https://node-postgres.com/)
that integrates IAM Authentication for connecting JavaScript/TypeScript applications to Amazon Aurora DSQL clusters.

The Aurora DSQL Connector is designed as an authentication plugin that extends the functionality of the
node-postgres' Client and Pool to enable applications to authenticate with Amazon Aurora DSQL using IAM credentials.

## About the Connector

Amazon Aurora DSQL is a cloud-native distributed database with PostgreSQL compatibility. While it requires IAM authentication and time-bound tokens, traditional Node.js database drivers lack this built-in support.

The Aurora DSQL Connector for node-postgres bridges this gap by implementing an authentication middleware that works seamlessly with node-postgres. This approach allows developers to maintain their existing node-postgres code while gaining secure IAM-based access to Aurora DSQL clusters through automated token management.

### What is Aurora DSQL Authentication?

In Aurora DSQL, authentication involves:

- **IAM Authentication:** All connections use IAM-based authentication with time-limited tokens
- **Token Generation:** Authentication tokens are generated using AWS credentials and have configurable lifetimes

The Aurora DSQL Connector for node-postgres is designed to understand these requirements and automatically generate IAM authentication tokens when establishing connections.

### Features

- **Automatic IAM Authentication** - Handles DSQL token generation and refresh
- **Built on node-postgres** - Leverages the popular PostgreSQL client for Node.js
- **Seamless Integration** - Works with existing node-postgres connection patterns
- **Region Auto-Discovery** - Extracts AWS region from DSQL cluster hostname
- **Full TypeScript Support** - Provides full type safety
- **AWS Credentials Support** - Supports various AWS credential providers (default, profile-based, custom)
- **Connection Pooling Compatibility** - Works seamlessly with built-in connection pooling
- **OCC Retry** - Automatic retry with jitter for optimistic concurrency control conflicts

## Example Application

There is an included sample application in [example](https://github.com/awslabs/aurora-dsql-connectors/tree/main/node/node-postgres/example) that shows how to use Aurora DSQL Connector for node-postgres. To run the included example please refer to the example [README](https://github.com/awslabs/aurora-dsql-connectors/blob/main/node/node-postgres/example/README.md).

## Quick start guide

### Requirements

- Node.js 20+
- [Access to an Aurora DSQL cluster](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/getting-started.html)
- Set up appropriate IAM permissions to allow your application to connect to Aurora DSQL.
- AWS credentials configured (via AWS CLI, environment variables, or IAM roles)

## ⚠️ Important

* Running this code might result in charges to your AWS account.
* We recommend that you grant your code least privilege. At most, grant only the
  minimum permissions required to perform the task. For more information, see
  [Grant least privilege](https://docs.aws.amazon.com/IAM/latest/UserGuide/best-practices.html#grant-least-privilege).
* This code is not tested in every AWS Region. For more information, see
  [AWS Regional Services](https://aws.amazon.com/about-aws/global-infrastructure/regional-product-services).

## Installation

```bash
npm install @aws/aurora-dsql-node-postgres-connector
```

## Peer Dependencies

```bash
npm install @aws-sdk/credential-providers @aws-sdk/dsql-signer pg tsx
npm install --save-dev @types/pg
```

## Usage

### Client Connection

```typescript
// src/index.ts
import { AuroraDSQLClient } from "@aws/aurora-dsql-node-postgres-connector";

const client = new AuroraDSQLClient({
  host: "<CLUSTER_ENDPOINT>",
  user: "admin",
});
await client.connect();
const result = await client.query("SELECT NOW()");
await client.end();
```

### Pool Connection

```typescript
// src/index.ts
import { AuroraDSQLPool } from "@aws/aurora-dsql-node-postgres-connector";

const pool = new AuroraDSQLPool({
  host: "<CLUSTER_ENDPOINT>",
  user: "admin",
  max: 3,
  idleTimeoutMillis: 60000,
});

const result = await pool.query("SELECT NOW()");
```

### Advanced Usage

```typescript
// index.ts
import { fromNodeProviderChain } from "@aws-sdk/credential-providers";
import { AuroraDSQLClient } from "@aws/aurora-dsql-node-postgres-connector";

const client = new AuroraDSQLClient({
  host: "example.dsql.us-east-1.on.aws",
  user: "admin",
  customCredentialsProvider: fromNodeProviderChain(), // Optionally provide custom credentials provider
});

await client.connect();
const result = await client.query("SELECT NOW()");
await client.end();
```

## Configuration Options

| Option                      | Type                                                    | Required | Description                                              |
| --------------------------- | ------------------------------------------------------- | -------- | -------------------------------------------------------- |
| `host`                      | `string`                                                | Yes      | DSQL cluster hostname                                    |
| `username`                  | `string`                                                | Yes      | DSQL username                                            |
| `database`                  | `string`                                                | No       | Database name                                            |
| `region`                    | `string`                                                | No       | AWS region (auto-detected from hostname if not provided) |
| `port`                      | `number`                                                | No       | Default to 5432                                          |
| `customCredentialsProvider` | `AwsCredentialIdentity / AwsCredentialIdentityProvider` | No       | Custom AWS credentials provider                          |
| `profile`                   | `string`                                                | No       | The IAM profile name. Default to "default"               |
| `tokenDurationSecs`         | `number`                                                | No       | Token expiration time in seconds                         |
| `logger`                    | `(msg: string) => void`                                 | No       | Optional callback for connector diagnostics              |
| `transaction`               | `{ retry?: RetryConfig }`                               | No       | Default retry config for `transaction()` calls           |

All other parameters from [Client](https://node-postgres.com/apis/client) / [Pool](https://node-postgres.com/apis/pool) are supported.

## Authentication

The connector automatically handles DSQL authentication by generating tokens using the DSQL client token generator. If the AWS region is not provided, it will be automatically parsed from the hostname provided.

For more information on authentication in Aurora DSQL, see the [user guide](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/authentication-authorization.html).

### Admin vs Regular Users

- Users named "admin" automatically use admin authentication tokens
- All other users use regular authentication tokens
- Tokens are generated dynamically for each connection

## OCC Retry

Aurora DSQL uses optimistic concurrency control (OCC). Transactions may fail with OCC errors when concurrent modifications conflict. The connector provides a `transaction()` method on both `AuroraDSQLPool` and `AuroraDSQLClient` that automatically detects and retries these conflicts.

### Using Pool (Recommended)

```typescript
import { AuroraDSQLPool } from "@aws/aurora-dsql-node-postgres-connector";

const pool = new AuroraDSQLPool({
  host: "<CLUSTER_ENDPOINT>",
  user: "admin",
  transaction: {
    retry: { maxAttempts: 5, baseDelayMs: 10 },
  },
});

// Transactions are automatically retried on OCC conflict
const result = await pool.transaction(async (client) => {
  await client.query("UPDATE accounts SET balance = balance - $1 WHERE id = $2", [100, fromId]);
  await client.query("UPDATE accounts SET balance = balance + $1 WHERE id = $2", [100, toId]);
  return client.query("SELECT balance FROM accounts WHERE id = $1", [fromId]);
});
```

### Using Client

```typescript
import { AuroraDSQLClient } from "@aws/aurora-dsql-node-postgres-connector";

const client = new AuroraDSQLClient({
  host: "<CLUSTER_ENDPOINT>",
  user: "admin",
  transaction: {
    retry: { maxAttempts: 5},
  },
});
await client.connect();

const result = await client.transaction(async (c) => {
  await c.query("INSERT INTO users (id, name) VALUES (gen_random_uuid(), $1)", ["Alice"]);
  return c.query("SELECT * FROM users WHERE name = $1", ["Alice"]);
});
```

**Opting Out:** For operations that don't need retry, use node-postgres directly:

```typescript
// Direct usage - no OCC retry
await client.query("BEGIN");
await client.query("SELECT * FROM users");
await client.query("COMMIT");
```

### OCC Configuration

Retry options can be set at the constructor level (shown above) or overridden per-call:

| Option         | Type      | Default | Description                              |
|----------------|-----------|---------|------------------------------------------|
| `maxAttempts`  | `number`  | `3`     | Total attempts (must be a positive integer) |
| `baseDelayMs`  | `number`  | `1`     | Base delay between retries (ms)          |
| `maxDelayMs`   | `number`  | `100`   | Maximum delay cap (ms)                   |
| `jitter`       | `boolean` | `true`  | Randomize delay to reduce contention     |

```typescript
// Per-call override
await pool.transaction(callback, {
  retry: { maxAttempts: 10 },
});

// Disable retry for a single call
await pool.transaction(callback, { retry: false });
```

**Backoff Strategy:**
- Jittered: `delay = baseDelayMs + random(0..1) * baseDelayMs`
- Capped at `maxDelayMs`
- When jitter is disabled, delay is constant at `baseDelayMs`

### OCC Error Types

The connector classifies OCC errors by type:

| SQLSTATE | Type     | Description                                              |
|----------|----------|----------------------------------------------------------|
| `OC000`  | Data     | Data conflict - concurrent modification of same rows     |
| `OC001`  | Schema   | Schema conflict - DDL changes during transaction         |
| `40001`  | Unknown  | Generic serialization failure (parsed for embedded OC000/OC001) |

Non-OCC errors are not retried and propagate immediately.

### Logging

The `logger` option is optional. If enabled, the connector sends OCC retry logs to your function. You can also customize the output:

```typescript
const pool = new AuroraDSQLPool({
  host: "<CLUSTER_ENDPOINT>",
  user: "admin",
  logger: (msg) => console.log(`[pool] ${msg}`),
  transaction: { retry: { maxAttempts: 5 } },
});
```

Output:
```
[pool] OCC conflict (Data) on attempt 1, retrying in 1.4ms
[pool] OCC retry exhausted after 5 attempts (Data conflict)
```

## Development

```
# Install dependencies
npm install

# Build the project
npm run build

# Set a cluster for use in the tests
export CLUSTER_ENDPOINT=your-cluster.dsql.us-east-1.on.aws

# Run tests
npm run test

```

## License

This software is released under the Apache 2.0 license.

Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
SPDX-License-Identifier: Apache-2.0
