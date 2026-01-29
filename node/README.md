# Aurora DSQL Connectors for Node.js

[![GitHub](https://img.shields.io/badge/github-awslabs/aurora--dsql--nodejs--connector-blue?logo=github)](https://github.com/awslabs/aurora-dsql-nodejs-connector)
[![License](https://img.shields.io/badge/license-Apache--2.0-brightgreen)](https://github.com/awslabs/aurora-dsql-nodejs-connector/blob/main/LICENSE)
[![Discord chat](https://img.shields.io/discord/1435027294837276802.svg?logo=discord)](https://discord.com/invite/nEF6ksFWru)

This repository contains Node.js connectors for [Amazon Aurora DSQL](https://aws.amazon.com/rds/aurora/dsql/), a distributed SQL database service that provides high availability and scalability for PostgreSQL-compatible applications.

Aurora DSQL requires IAM-based authentication with time-limited tokens that existing Node.js PostgreSQL drivers do not natively support. These connectors bridge that gap by providing seamless IAM authentication on top of popular PostgreSQL clients.

## Available Connectors

### [Aurora DSQL Connector for node-postgres](./packages/node-postgres/)

Built on [node-postgres](https://node-postgres.com/), the most popular PostgreSQL client for Node.js. This connector extends the node-postgres Client and Pool with automatic IAM authentication for Aurora DSQL.

[View Documentation](./packages/node-postgres/README.md)

[Sample Usage](./packages/node-postgres/example/)

### [Aurora DSQL Connector for Postgres.js](./packages/postgres-js/)

Built on [Postgres.js](https://github.com/porsager/postgres), a fast and modern PostgreSQL client. This connector provides IAM authentication while maintaining Postgres.js's tagged template literal query syntax.

[View Documentation](./packages/postgres-js/README.md)

[Sample Usage](./packages/postgres-js/example/)

## Key Features

Both connectors provide:

- **Automatic IAM Authentication** - Handles DSQL token generation and refresh
- **Seamless Integration** - Built on popular PostgreSQL clients to support existing PostgreSQL applications
- **Region Auto-Discovery** - Extracts AWS region from DSQL cluster hostname  
- **Full TypeScript Support** - Complete type safety for your applications
- **Custom Credentials** - Support for custom AWS credential providers

## Prerequisites

- Node.js 20+
- AWS credentials configured (via AWS CLI, environment variables, or IAM roles)
- Access to an Aurora DSQL cluster

## ⚠️ Important

* Running this code might result in charges to your AWS account.
* We recommend that you grant your code least privilege. At most, grant only the
  minimum permissions required to perform the task. For more information, see
  [Grant least privilege](https://docs.aws.amazon.com/IAM/latest/UserGuide/best-practices.html#grant-least-privilege).
* This code is not tested in every AWS Region. For more information, see
  [AWS Regional Services](https://aws.amazon.com/about-aws/global-infrastructure/regional-product-services).

## Contributing

See [CONTRIBUTING](CONTRIBUTING.md#security-issue-notifications) for more information.

## License

This project is licensed under the Apache License 2.0. See [LICENSE](./LICENSE) for details.
