# Aurora DSQL Connector for Postgres.js

[![GitHub](https://img.shields.io/badge/github-awslabs/aurora--dsql--postgres--js--connector-blue?logo=github)](https://github.com/awslabs/aurora-dsql-nodejs-connector/tree/main/packages/postgres-js)
[![License](https://img.shields.io/badge/license-Apache--2.0-brightgreen)](https://github.com/awslabs/aurora-dsql-nodejs-connector/blob/main/LICENSE)
[![NPM Version](https://img.shields.io/npm/v/@aws/aurora-dsql-postgresjs-connector)](https://www.npmjs.com/package/@aws/aurora-dsql-postgresjs-connector)
[![Discord chat](https://img.shields.io/discord/500028886025895936.svg?logo=discord)](https://discord.com/invite/nEF6ksFWru)

The Aurora DSQL Connector for Postgres.js is a Node.js connector built on [Postgres.js](https://github.com/porsager/postgres) 
that integrates IAM Authentication for connecting JavaScript applications to Amazon Aurora DSQL clusters.

The Aurora DSQL Connector for Postgres.js is designed as an authentication plugin that extends the functionality of the
Postgres.js client to enable applications to authenticate with Amazon Aurora DSQL using IAM credentials. The connector 
does not connect directly to the database, but provides seamless IAM authentication on top of the underlying Postgres.js driver.

## Benefits of the Connector

Amazon Aurora DSQL is a distributed SQL database service that provides high availability and scalability for 
PostgreSQL-compatible applications. Aurora DSQL requires IAM-based authentication with time-limited tokens that 
existing Node.js drivers do not natively support.

The idea behind the Aurora DSQL Connector for Postgres.js is to add an authentication layer on top of the Postgres.js 
client that handles IAM token generation, allowing users to connect to Aurora DSQL without changing their existing Postgres.js workflows.

The Aurora DSQL Connector for Postgres.js works with most versions of Postgres.js. Users provide their own version by installing
Postgres.js directly.

### Features

- **Automatic IAM Authentication** - Handles DSQL token generation and refresh
- **Built on Postgres.js** - Leverages the fast PostgreSQL client for Node.js
- **Region Auto-Discovery** - Extracts AWS region from DSQL cluster hostname
- **Full TypeScript Support** - Provides full type safety
- **Custom Credentials** - Support for custom AWS credential providers

## Quick start guide

### Requirements

- Node.js 20+
- AWS credentials configured (via AWS CLI, environment variables, or IAM roles)
- Access to an Aurora DSQL cluster

### Installation

```bash
npm install @aws/aurora-dsql-postgresjs-connector
# Postgres.js is a peer-dependency, so users must install it themselves
npm install postgres
```

### Basic Usage

```typescript
import { auroraDSQLPostgres } from '@aws/aurora-dsql-postgresjs-connector';

const sql = auroraDSQLPostgres({
  host: 'your-cluster.dsql.us-east-1.on.aws',
  username: 'admin',
    
});

// Execute queries
const users = await sql`SELECT * FROM users WHERE age > ${25}`;
console.log(users);

// Clean up
await sql.end();
```

#### Using cluster ID instead of host

```typescript
const sql = auroraDSQLPostgres({
  host: 'your-cluster-id',
  region: 'us-east-1',
  username: 'admin',
    
});
```

### Connection String

```typescript
const sql = AuroraDSQLPostgres(
  'postgres://admin@your-cluster.dsql.us-east-1.on.aws'
);

const result = await sql`SELECT current_timestamp`;
```

### Advanced Configuration

```typescript
import { fromNodeProviderChain } from '@aws-sdk/credential-providers';

const sql = AuroraDSQLPostgres({
  host: 'your-cluster.dsql.us-east-1.on.aws',
  database: 'postgres',
  username: 'admin',
  customCredentialsProvider: fromNodeProviderChain(), // Optionally provide custom credentials provider
  tokenDurationSecs: 3600,                            // Token expiration (seconds)
  
  // Standard Postgres.js options
  max: 20,                              // Connection pool size
  ssl: { rejectUnauthorized: false }    // SSL configuration
});
```

## Configuration Options

| Option                      | Type                             | Required | Description                                              |
|-----------------------------|----------------------------------|----------|----------------------------------------------------------|
| `host`                      | `string`                         | Yes      | DSQL cluster hostname or cluster ID                      |
| `database`                  | `string?`                        | No       | Database name                                            |
| `username`                  | `string?`                        | No       | Database username (uses admin if not provided)           |
| `region`                    | `string?`                        | No       | AWS region (auto-detected from hostname if not provided) |
| `customCredentialsProvider` | `AwsCredentialIdentityProvider?` | No       | Custom AWS credentials provider                          |
| `tokenDurationSecs`         | `number?`                        | No       | Token expiration time in seconds                         |

All standard [Postgres.js options](https://github.com/porsager/postgres?tab=readme-ov-file#connection-details) are also supported.

## Authentication

The connector automatically handles DSQL authentication by generating tokens using the DSQL client token generator. If the
AWS region is not provided, it will be automatically parsed from the hostname provided.

For more information on authentication in Aurora DSQL, see the [user guide](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/authentication-authorization.html).

### Admin vs Regular Users

- Users named `"admin"` automatically use admin authentication tokens
- All other users use regular authentication tokens
- Tokens are generated dynamically for each connection

## Sample usage

An JavaScript example using the Aurora DSQL Connector for Postgres.js is available [here](example).

## Development

```bash
# Install dependencies
npm install

# Build the project
npm run build

# Set a cluster for use in integration tests
export CLUSTER_ENDPOINT=your-cluster.dsql.us-east-1.on.aws

# Run tests
npm run test

# Alternatively, run only unit or integration tests
npm run test:unit
npm run test:integration
```

## License
This software is released under the Apache 2.0 license.

Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
SPDX-License-Identifier: Apache-2.0
