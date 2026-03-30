# Aurora DSQL with SQLx

## Overview

This code example demonstrates how to use SQLx with Amazon Aurora DSQL.
The example shows you how to connect to an Aurora DSQL cluster and perform basic database operations.

Aurora DSQL is a distributed SQL database service that provides high availability and scalability for
your PostgreSQL-compatible applications. SQLx is a popular async SQL toolkit for Rust that allows
you to interact with PostgreSQL databases using Rust code.

This example uses the Aurora DSQL SQLx Connector to handle IAM authentication automatically.

## About the code example

This directory contains multiple examples. The **preferred example** (`example_preferred`) demonstrates a flexible
connection approach that works for both admin and non-admin users:

* When connecting as an **admin user**, `example_preferred` uses the `public` schema.
* When connecting as a **non-admin user**, `example_preferred` uses a custom `myschema` schema.

In `example_preferred`, the code automatically detects the user type and adjusts its behavior accordingly.

The **no connection pool example** (`example_no_connection_pool`) demonstrates simpler single-connection usage without
pooling or schema detection.

## ⚠️ Important

* Running this code might result in charges to your AWS account.
* We recommend that you grant your code least privilege. At most, grant only the
  minimum permissions required to perform the task. For more information, see
  [Grant least privilege](https://docs.aws.amazon.com/IAM/latest/UserGuide/best-practices.html#grant-least-privilege).
* This code is not tested in every AWS Region. For more information, see
  [AWS Regional Services](https://aws.amazon.com/about-aws/global-infrastructure/regional-product-services).

## TLS connection configuration

This example uses rustls-based TLS for connections and verifies that the server certificate is trusted and the hostname is valid.
Verified TLS connections should be used where possible to ensure data security during transmission.

* The connector uses rustls for TLS with ring cryptography (`tls-rustls-ring` feature)
* The connector enforces certificate and hostname verification (equivalent to `ssl_mode=VerifyFull`)
* The connector automatically configures TLS parameters for Aurora DSQL connections
* TLS is required by the connector when establishing Aurora DSQL connections

## Run the example

### Prerequisites

* You must have an AWS account, and have your default credentials and AWS Region
  configured as described in the
  [Globally configuring AWS SDKs and tools](https://docs.aws.amazon.com/credref/latest/refdocs/creds-config-files.html)
  guide.
* Rust: Ensure you have Rust 1.80+ installed.

   _To verify Rust is installed, you can run_
   ```bash
   rustc --version
   ```

* You must have an Aurora DSQL cluster. For information about creating an Aurora DSQL cluster, see the
  [Getting started with Aurora DSQL](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/getting-started.html)
  guide.
* If connecting as a non-admin user, ensure the user is linked to an IAM role and is granted access to the `myschema`
  schema. See the
  [Using database roles with IAM roles](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/using-database-and-iam-roles.html)
  guide.

### Run the code

The **preferred example** demonstrates the following operations:

- Opening a connection pool to an Aurora DSQL cluster
- Creating a table
- Performing a transactional insert with OCC retry
- Running concurrent queries across multiple tokio tasks

The preferred example is designed to work with both admin and non-admin users:

- When run as an admin user, it uses the `public` schema
- When run as a non-admin user, it uses the `myschema` schema

The **no connection pool example** demonstrates simpler single-connection usage without automatic schema detection.

**Note:** running the example will use actual resources in your AWS account and may incur charges.

Set environment variables for your cluster details:

```bash
# e.g. "admin"
export CLUSTER_USER="<your user>"

# e.g. "foo0bar1baz2quux3quuux4.dsql.us-east-1.on.aws"
export CLUSTER_ENDPOINT="<your endpoint>"
```

Run the preferred example (connection pool with OCC retry):

```bash
cargo run --bin example_preferred
```

Run the no connection pool example:

```bash
cargo run --bin example_no_connection_pool
```

Run the tests:

```bash
cargo test
```

The example contains comments explaining the code and the operations being performed.

## Additional resources

* [Amazon Aurora DSQL Documentation](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/what-is-aurora-dsql.html)
* [Amazon Aurora DSQL SQLx Connector](https://github.com/awslabs/aurora-dsql-connectors/tree/main/rust/sqlx)
* [SQLx Documentation](https://docs.rs/sqlx/latest/sqlx/)
* [AWS SDK for Rust Documentation](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html)

---

Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.

SPDX-License-Identifier: Apache-2.0
