# Aurora DSQL Connector for node-postgres

[![GitHub](https://img.shields.io/badge/github-awslabs/aurora--dsql--node--postgres--connector-blue?logo=github)](https://github.com/awslabs/aurora-dsql-nodejs-connector/tree/main/packages/node-postgres)
[![License](https://img.shields.io/badge/license-Apache--2.0-brightgreen)](https://github.com/awslabs/aurora-dsql-nodejs-connector/blob/main/LICENSE)
[![NPM Version](https://img.shields.io/npm/v/@aws/aurora-dsql-node-postgres-connector)](https://www.npmjs.com/package/@aws/aurora-dsql-node-postgres-connector)
[![Discord chat](https://img.shields.io/discord/1435027294837276802.svg?logo=discord)](https://discord.com/invite/nEF6ksFWru)

The Aurora DSQL Connector for node-postgres is a Node.js connector built on [node-postgres](https://node-postgres.com/)
that integrates IAM Authentication for connecting JavaScript/TypeScript applications to Amazon Aurora DSQL clusters.

The Aurora DSQL Connector is designed as an authentication plugin that extends the functionality of the
node-postgres' Client and Pool to enable applications to authenticate with Amazon Aurora DSQL using IAM credentials.

## About the Connector

Amazon Aurora DSQL is a cloud-native distributed database with PostgreSQL compatibility. While it requires IAM authentication and time-bound tokens, traditional Node.js database drivers lack this built-in support.

The Aurora DSQL Connector for node-postgres bridges this gap by implementing an authentication middleware that works seamlessly with node-postgres. This approach allows developers to maintain their existing node-postgres code while gaining secure IAM-based access to Aurora DSQL clusters through automated token management.

### Features

- **Automatic IAM Authentication** - Handles DSQL token generation and refresh
- **Built on node-postgres** - Leverages the popular PostgreSQL client for Node.js
- **Region Auto-Discovery** - Extracts AWS region from DSQL cluster hostname
- **Full TypeScript Support** - Provides full type safety
- **Custom Credentials** - Support for custom AWS credential providers

## Example Application

There is an included sample application in [example](https://github.com/awslabs/aurora-dsql-nodejs-connector/tree/main/packages/node-postgres/example) that shows how to use Aurora DSQL Connector for node-postgres. To run the included example please refer to the example [README](https://github.com/awslabs/aurora-dsql-nodejs-connector/blob/main/packages/node-postgres/example/README.md).

## Quick start guide

### Requirements

- Node.js 20+
- AWS credentials configured (via AWS CLI, environment variables, or IAM roles)
- Access to an Aurora DSQL cluster

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

All other parameters from [Client](https://node-postgres.com/apis/client) / [Pool](https://node-postgres.com/apis/pool) are supported.

## Authentication

The connector automatically handles DSQL authentication by generating tokens using the DSQL client token generator. If the AWS region is not provided, it will be automatically parsed from the hostname provided.

For more information on authentication in Aurora DSQL, see the [user guide](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/authentication-authorization.html).

### Admin vs Regular Users

- Users named "admin" automatically use admin authentication tokens
- All other users use regular authentication tokens
- Tokens are generated dynamically for each connection

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
