# Aurora DSQL Connectors

This monorepo contains database driver connectors for [Amazon Aurora DSQL](https://aws.amazon.com/rds/aurora/dsql/), AWS's distributed SQL database.

## Available Connectors

### Go

| Package | Description | Module | License |
|---------|-------------|--------|---------|
| [aurora-dsql-pgx-connector](./go/pgx/) | pgx connector for Aurora DSQL | `github.com/awslabs/aurora-dsql-connectors/go/pgx` | ![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg) |

### Java

| Package | Description | Maven Central | License |
|---------|-------------|---------------|---------|
| [aurora-dsql-jdbc-connector](./java/jdbc/) | JDBC connector for Aurora DSQL | [![Maven Central](https://img.shields.io/maven-central/v/software.amazon.dsql/aurora-dsql-jdbc-connector)](https://central.sonatype.com/artifact/software.amazon.dsql/aurora-dsql-jdbc-connector) | ![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg) |

### Node.js

| Package | Description | npm | License |
|---------|-------------|-----|---------|
| [@aws/aurora-dsql-node-postgres-connector](./node/node-postgres/) | node-postgres (pg) connector for Aurora DSQL | [![npm](https://img.shields.io/npm/v/@aws/aurora-dsql-node-postgres-connector)](https://www.npmjs.com/package/@aws/aurora-dsql-node-postgres-connector) | ![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg) |
| [@aws/aurora-dsql-postgresjs-connector](./node/postgres-js/) | Postgres.js connector for Aurora DSQL | [![npm](https://img.shields.io/npm/v/@aws/aurora-dsql-postgresjs-connector)](https://www.npmjs.com/package/@aws/aurora-dsql-postgresjs-connector) | ![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg) |

### Python

| Package | Description | PyPI | License |
|---------|-------------|------|---------|
| [aurora-dsql-python-connector](./python/connector/) | Python connectors for Aurora DSQL (psycopg, psycopg2, asyncpg) | [![PyPI](https://img.shields.io/pypi/v/aurora-dsql-python-connector)](https://pypi.org/project/aurora-dsql-python-connector/) | ![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg) |

## Installation

Each connector is published as an independent package. Install the one you need:

```bash
# Python (with psycopg support)
pip install aurora-dsql-python-connector[psycopg]

# Python (with asyncpg support)
pip install aurora-dsql-python-connector[asyncpg]

# Node.js (node-postgres)
npm install @aws/aurora-dsql-node-postgres-connector

# Node.js (postgres.js)
npm install @aws/aurora-dsql-postgresjs-connector

# Go
go get github.com/awslabs/aurora-dsql-connectors/go/pgx
```

For Java connectors, see the [Java JDBC connector documentation](./java/jdbc/README.md) for Maven/Gradle installation instructions.

## Documentation

See the README in each connector's directory for detailed usage instructions:

- [Go pgx connector documentation](./go/pgx/README.md)
- [Java JDBC connector documentation](./java/jdbc/README.md)
- [Node.js node-postgres connector documentation](./node/node-postgres/README.md)
- [Node.js postgres.js connector documentation](./node/postgres-js/README.md)
- [Python connector documentation](./python/connector/README.md)

## Versioning

Each connector is versioned independently. Version numbers continue from the original standalone repositories to maintain backwards compatibility.

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines on how to contribute to this project.

## Security

See [SECURITY.md](./SECURITY.md) for information on reporting security issues.

## License

This repository is licensed under Apache-2.0 ([LICENSE](./LICENSE)).
