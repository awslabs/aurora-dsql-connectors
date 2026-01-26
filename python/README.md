# Aurora DSQL Connector for Python

[![GitHub](https://img.shields.io/badge/github-awslabs/aurora--dsql--python--connector-blue?logo=github)](https://github.com/awslabs/aurora-dsql-python-connector)
[![License](https://img.shields.io/badge/license-Apache--2.0-brightgreen)](https://github.com/awslabs/aurora-dsql-python-connector/blob/main/LICENSE)
[![PyPI - Version](https://img.shields.io/pypi/v/aurora-dsql-python-connector)](https://pypi.org/project/aurora-dsql-python-connector)
[![Discord chat](https://img.shields.io/discord/1435027294837276802.svg?logo=discord)](https://discord.com/invite/nEF6ksFWru)

The Aurora DSQL Connector for Python integrates IAM Authentication for connecting Python applications to Amazon Aurora DSQL clusters.
Internally, it utilizes [psycopg](https://github.com/psycopg/psycopg), [psycopg2](https://github.com/psycopg/psycopg2), and [asyncpg](https://github.com/MagicStack/asyncpg) client libraries.

The Aurora DSQL Connector for Python is designed as an authentication plugin that extends the functionality of the psycopg, psycopg2, and asyncpg
client libraries to enable applications to authenticate with Amazon Aurora DSQL using IAM credentials. The connector 
does not connect directly to the database but provides seamless IAM authentication on top of the underlying client libraries.

## About the Connector

Amazon Aurora DSQL is a distributed SQL database service that provides high availability and scalability for 
PostgreSQL-compatible applications. Aurora DSQL requires IAM-based authentication with time-limited tokens that 
existing Python libraries do not natively support.

The idea behind the Aurora DSQL Connector for Python is to add an authentication layer on top of the psycopg, psycopg2, and asyncpg
client libraries that handles IAM token generation, allowing users to connect to Aurora DSQL without changing their existing workflows.

### What is Aurora DSQL Authentication?

In Aurora DSQL, authentication involves:

- **IAM Authentication:** All connections use IAM-based authentication with time-limited tokens
- **Token Generation:** Authentication tokens are generated using AWS credentials and have configurable lifetimes

The Aurora DSQL Connector for Python is designed to understand these requirements and automatically generate IAM authentication tokens when establishing connections.


### Features

- **Automatic IAM Authentication** - IAM tokens are generated automatically using AWS credentials
- **Built on psycopg, psycopg2, and asyncpg** - Leverages the psycopg, psycopg2, and asyncpg client libraries
- **Seamless Integration** - Works with existing psycopg, psycopg2, and asyncpg connection patterns without requiring workflow changes
- **Region Auto-Discovery** - Extracts AWS region from DSQL cluster hostname
- **AWS Credentials Support** - Supports various AWS credential providers (default, profile-based, custom)
- **Connection Pooling Compatibility** - Works with psycopg, psycopg2, and asyncpg built-in connection pooling

## Quick start guide

### Requirements

- Python 3.10 or higher
- [Access to an Aurora DSQL cluster](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/getting-started.html)
- Set up appropriate IAM permissions to allow your application to connect to Aurora DSQL.
- AWS credentials configured (via AWS CLI, environment variables, or IAM roles)


### Installation

```bash
pip install aurora-dsql-python-connector
```

#### Install psycopg or psycopg2 or asyncpg separately

The Aurora DSQL Connector for Python installer does not install the underlying libraries.
They need to be installed separately, e.g.:

```bash
# Install psycopg and psycopg pool
pip install "psycopg[binary,pool]"
```

```bash
# Install psycopg2
pip install psycopg2-binary
```

```bash
# Install asyncpg
pip install asyncpg
```

**Note:**

Only the library that is needed must be installed.
Therefore, if the client is going to use psycopg, then only psycopg needs to be installed.
If the client is going to use psycopg2, then only psycopg2 needs to be installed.
If the client is going to use asyncpg, then only asyncpg needs to be installed.

If the client needs more than one, then all the needed libraries need to be installed.

### Basic Usage 

#### psycopg

```python
    import aurora_dsql_psycopg as dsql

    config = {
        'host': "your-cluster.dsql.us-east-1.on.aws",
        'region': "us-east-1",
        'user': "admin",
    }
        
    conn = dsql.connect(**config)
    with conn.cursor() as cur:
        cur.execute("SELECT 1")
        result = cur.fetchone()
        print(result)
```

#### psycopg2

```python
    import aurora_dsql_psycopg2 as dsql

    config = {
        'host': "your-cluster.dsql.us-east-1.on.aws",
        'region': "us-east-1",
        'user': "admin",
    }
        
    conn = dsql.connect(**config)
    with conn.cursor() as cur:
        cur.execute("SELECT 1")
        result = cur.fetchone()
        print(result)
```

#### asyncpg

```python
    import asyncio 
    import aurora_dsql_asyncpg as dsql 

    config = {
        'host': "your-cluster.dsql.us-east-1.on.aws",
        'region': "us-east-1",
        'user': "admin",
    }

    conn = await dsql.connect(**config)
    result = await conn.fetchrow("SELECT 1")
    await conn.close()
    print(result)
```

#### Using just host

##### psycopg

```python
    import aurora_dsql_psycopg as dsql

    conn = dsql.connect("your-cluster.dsql.us-east-1.on.aws")
```

##### psycopg2

```python
    import aurora_dsql_psycopg2 as dsql

    conn = dsql.connect("your-cluster.dsql.us-east-1.on.aws")
```

##### asyncpg

```python
    import asyncio 
    import aurora_dsql_asyncpg as dsql 

    conn = await dsql.connect("your-cluster.dsql.us-east-1.on.aws")
```


#### Using just cluster ID

##### psycopg

```python
    import aurora_dsql_psycopg as dsql

    conn = dsql.connect("your-cluster")
```

##### psycopg2

```python
    import aurora_dsql_psycopg2 as dsql

    conn = dsql.connect("your-cluster")
```

##### asyncpg

```python
    import asyncio 
    import aurora_dsql_asyncpg as dsql 

    conn = await dsql.connect("your-cluster")
```

**Note:** 

In the 'using just cluster ID' scenario, the region that was set previously on the machine is used, e.g.:

```bash
aws configure set region us-east-1
```

If the region has not been set, or the given cluster ID is in a different region, the connection will fail.
To make it work, provide region as a parameter as in the example below:

##### psycopg

```python
    import aurora_dsql_psycopg as dsql

    config = {
            "region": "us-east-1",
    }

    conn = dsql.connect("your-cluster", **config)
```

##### psycopg2

```python
    import aurora_dsql_psycopg2 as dsql

    config = {
            "region": "us-east-1",
    }

    conn = dsql.connect("your-cluster", **config)
```

##### asyncpg

```python
    import asyncio 
    import aurora_dsql_asyncpg as dsql 

    config = {
            "region": "us-east-1",
    }

    conn = await dsql.connect("your-cluster", **config)
```

### Connection String

#### psycopg

```python
    import aurora_dsql_psycopg as dsql

    conn = dsql.connect("postgresql://your-cluster.dsql.us-east-1.on.aws/postgres?user=admin&token_duration_secs=15")
```

#### psycopg2

```python
    import aurora_dsql_psycopg2 as dsql

    conn = dsql.connect("postgresql://your-cluster.dsql.us-east-1.on.aws/postgres?user=admin&token_duration_secs=15")
```

#### asyncpg

```python
    import asyncio 
    import aurora_dsql_asyncpg as dsql 

    conn = await dsql.connect("postgresql://your-cluster.dsql.us-east-1.on.aws/postgres?user=admin&token_duration_secs=15")
```

### Advanced Configuration

#### psycopg

```python
    import aurora_dsql_psycopg as dsql

    config = {
        'host': "your-cluster.dsql.us-east-1.on.aws",
        'region': "us-east-1",
        'user': "admin",
        "profile": "default",
        "token_duration_secs": "15",
    }
        
    conn = dsql.connect(**config)
    with conn.cursor() as cur:
        cur.execute("SELECT 1")
        result = cur.fetchone()
        print(result)
```

#### psycopg2

```python
    import aurora_dsql_psycopg2 as dsql

    config = {
        'host': "your-cluster.dsql.us-east-1.on.aws",
        'region': "us-east-1",
        'user': "admin",
        "profile": "default",
        "token_duration_secs": "15",
    }
        
    conn = dsql.connect(**config)
    with conn.cursor() as cur:
        cur.execute("SELECT 1")
        result = cur.fetchone()
        print(result)
```

#### asyncpg

```python
    import asyncio 
    import aurora_dsql_asyncpg as dsql 

    config = {
        'host': "your-cluster.dsql.us-east-1.on.aws",
        'region': "us-east-1",
        'user': "admin",
        "profile": "default",
        "token_duration_secs": "15",
    }

    conn = await dsql.connect(**config)
    result = await conn.fetchrow("SELECT 1")
    await conn.close()
    print(result)
```

### Configuration Options

| Option                        | Type                     | Required | Description                                                   |
|-------------------------------|--------------------------|----------|---------------------------------------------------------------|
| `host`                        | `string`                 | Yes      | DSQL cluster hostname or cluster ID                           |
| `user`                        | `string`                 | No       | DSQL username. Default: admin                                 |
| `dbname`                      | `string`                 | No       | Database name.  Default: postgres                             |
| `region`                      | `string`                 | No       | AWS region (auto-detected from hostname if not provided)      |
| `port`                        | `int`                    | No       | Default to 5432                                               |
| `custom_credentials_provider` | `CredentialProvider`     | No       | Custom AWS credentials provider                               |
| `profile`                     | `string`                 | No       | The IAM profile name. Default: default.                       |
| `token_duration_secs`         | `int`                    | No       | Token expiration time in seconds                              |


All standard connection options of the underlying psycopg, psycopg2, and asyncpg libraries are also supported, with 
the exception of asyncpg parameters **krbsrvname** and **gsslib** which are not supported by DSQL.

### Using the Aurora DSQL connector for Python with connection pooling

The Aurora DSQL Connector for Python works with psycopg, psycopg2, and asyncpg built-in connection pooling. The connector handles IAM token generation during connection establishment, allowing connection pools to operate normally.

#### psycopg

For psycopg, the connector implements a connection class named DSQLConnection that can be passed directly to the psycopg_pool.ConnectionPool constructor. For asynchronous operations, there is also an async version of the class named DSQLAsyncConnection.

```python
    from psycopg_pool import ConnectionPool as PsycopgPool
    
    ...
    pool = PsycopgPool(
        "",  
        connection_class=dsql.DSQLConnection,
        kwargs=conn_params,
        min_size=2,
        max_size=8,
        max_lifetime=3300
    )
```

**Note:  Connection max_lifetime Configuration**

The max_lifetime parameter should be set to less than 3600 seconds (one hour), as this is the maximum connection duration allowed by Aurora DSQL database. Setting a lower max_lifetime allows the connection pool to proactively manage connection recycling, which is more efficient than handling connection timeout errors from the database.

#### psycopg2 

For psycopg2, the connector provides a class named AuroraDSQLThreadedConnectionPool that inherits from psycopg2.pool.ThreadedConnectionPool. The AuroraDSQLThreadedConnectionPool class only overrides the internal _connect method. The rest of the implementation is provided by psycopg2.pool.ThreadedConnectionPool unchanged.

```python
    import aurora_dsql_psycopg2 as dsql

    pool = dsql.AuroraDSQLThreadedConnectionPool(
            minconn=2,
            maxconn=8,
            **conn_params,
    )

```

#### asyncpg

For asyncpg, the connector provides a create_pool function that returns an instance of asyncpg.Pool.

```python
    import asyncio
    import os

    import aurora_dsql_asyncpg as dsql

    pool_params = {
        'host': "your-cluster.dsql.us-east-1.on.aws",
        'user': "admin",
        "min_size": 2,
        "max_size": 5,
    }

    pool = await dsql.create_pool(**pool_params)
```



## Authentication

The connector automatically handles DSQL authentication by generating tokens using the DSQL client token generator. If the
AWS region is not provided, it will be automatically parsed from the hostname provided.

For more information on authentication in Aurora DSQL, see the [user guide](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/authentication-authorization.html).

### Admin vs Regular Users

- Users named `"admin"` automatically use admin authentication tokens
- All other users use non-admin authentication tokens
- Tokens are generated dynamically for each connection

## Examples

For full example code, refer to the examples as indicated in the sections below.
For instructions how to run the examples please refer to the examples READMDE files.

### psycopg

[Examples README](https://github.com/awslabs/aurora-dsql-python-connector/blob/main/examples/psycopg/README.md)

| Description                                                                     | Examples                                                                                                                                                                                   |
|---------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Using the Aurora DSQL Connector for Python for basic connections                | [Basic Connection Example](https://github.com/awslabs/aurora-dsql-python-connector/tree/main/examples/psycopg/src/example.py)                                                              |
| Using the Aurora DSQL Connector for Python for basic asynchronous connections   | [Basic Asynchronous Connection Example](https://github.com/awslabs/aurora-dsql-python-connector/tree/main/examples/psycopg/src/example_async.py)                                           |
| Using the Aurora DSQL Connector for Python with connection pool                 | [Basic Connection Example With Connection Pool](https://github.com/awslabs/aurora-dsql-python-connector/tree/main/examples/psycopg/src/example_with_connection_pool.py)                    |
|                                                                                 | [Concurrent Connections Example With Connection Pool](https://github.com/awslabs/aurora-dsql-python-connector/tree/main/examples/psycopg/src/example_with_connection_pool_concurrent.py)   |
| Using the Aurora DSQL Connector for Python with asynchronous connection pool    | [Basic Connection Example With Asynchronous Connection Pool](https://github.com/awslabs/aurora-dsql-python-connector/tree/main/examples/psycopg/src/example_with_connection_pool_async.py) |


### psycopg2 

[Examples README](https://github.com/awslabs/aurora-dsql-python-connector/blob/main/examples/psycopg2/README.md)

| Description                                                                     | Examples                                                                                                                                                                                   |
|---------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Using the Aurora DSQL Connector for Python for basic connections                | [Basic Connection Example](https://github.com/awslabs/aurora-dsql-python-connector/tree/main/examples/psycopg2/src/example.py)                                                             |
| Using the Aurora DSQL Connector for Python with connection pool                 | [Basic Connection Example With Connection Pool](https://github.com/awslabs/aurora-dsql-python-connector/tree/main/examples/psycopg2/src/example_with_connection_pool.py)                   |
|                                                                                 | [Concurrent Connections Example With Connection Pool](https://github.com/awslabs/aurora-dsql-python-connector/tree/main/examples/psycopg2/src/example_with_connection_pool_concurrent.py)  |


### asyncpg

[Examples README](https://github.com/awslabs/aurora-dsql-python-connector/blob/main/examples/asyncpg/README.md)

| Description                                                                     | Examples                                                                                                                                                                                   |
|---------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Using the Aurora DSQL Connector for Python for basic connections                | [Basic Connection Example](https://github.com/awslabs/aurora-dsql-python-connector/tree/main/examples/asyncpg/src/example.py)                                                              |
| Using the Aurora DSQL Connector for Python with connection pool                 | [Basic Connection Example With Connection Pool](https://github.com/awslabs/aurora-dsql-python-connector/tree/main/examples/asyncpg/src/example_with_connection_pool.py)                    |
|                                                                                 | [Concurrent Connections Example With Connection Pool](https://github.com/awslabs/aurora-dsql-python-connector/tree/main/examples/asyncpg/src/example_with_connection_pool_concurrent.py)   |



## Development

### Install `uv`

Install `uv` using the [official installation guide](https://docs.astral.sh/uv/getting-started/installation/) or via [mise](https://mise.jdx.dev/).

### Install dependencies

```bash
uv sync
```

### Install pre-commit hooks

```bash
uv run pre-commit install
```

### Run unit tests

```bash
uv run pytest tests/unit
```

### Configure environment variables

Copy `.env.example` to `.env` and update with your cluster details:

```bash
cp .env.example .env
```

Alternatively, set the environment variable directly:

```bash
export CLUSTER_ENDPOINT=your-cluster.dsql.us-east-1.on.aws
```

### Run integration tests

```bash
uv run pytest tests/integration
```

## License
This software is released under the Apache 2.0 license.

Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
SPDX-License-Identifier: Apache-2.0
