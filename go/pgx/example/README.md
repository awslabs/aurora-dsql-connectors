# Aurora DSQL with pgx

## Overview

This code example demonstrates how to use `pgx` with Amazon Aurora DSQL.
The example shows you how to connect to an Aurora DSQL cluster and perform basic database operations.

Aurora DSQL is a distributed SQL database service that provides high availability and scalability for
your PostgreSQL-compatible applications. `pgx` is a popular PostgreSQL driver for Go that allows
you to interact with PostgreSQL databases using Go code.

This example uses the Aurora DSQL pgx Connector to handle IAM authentication automatically.

## About the code example

The example demonstrates a flexible connection approach that works for both admin and non-admin users:

* When connecting as an **admin user**, the example uses the `public` schema.
* When connecting as a **non-admin user**, the example uses a custom `myschema` schema.

The code automatically detects the user type and adjusts its behavior accordingly.

It creates a connection pool via `dsql.NewPool`, verifies connectivity, and runs multiple
concurrent workers that each execute queries through the pool.

## ⚠️ Important

* Running this code might result in charges to your AWS account.
* We recommend that you grant your code least privilege. At most, grant only the
  minimum permissions required to perform the task. For more information, see
  [Grant least privilege](https://docs.aws.amazon.com/IAM/latest/UserGuide/best-practices.html#grant-least-privilege).
* This code is not tested in every AWS Region. For more information, see
  [AWS Regional Services](https://aws.amazon.com/about-aws/global-infrastructure/regional-product-services).

## TLS connection configuration

This example uses direct TLS connections where supported, and verifies the server certificate is trusted. Verified SSL
connections should be used where possible to ensure data security during transmission.

* Driver versions following the release of PostgreSQL 17 support direct TLS connections, bypassing the traditional
  PostgreSQL connection preamble
* Direct TLS connections provide improved connection performance and enhanced security
* Not all PostgreSQL drivers support direct TLS connections yet, or only in recent versions following PostgreSQL 17
* Ensure your installed driver version supports direct TLS negotiation, or use a version that is at least as recent as
  the one used in this sample
* If your driver doesn't support direct TLS connections, you may need to use the traditional preamble connection instead

## Run the example

### Prerequisites

* You must have an AWS account, and have your default credentials and AWS Region
  configured as described in the
  [Globally configuring AWS SDKs and tools](https://docs.aws.amazon.com/credref/latest/refdocs/creds-config-files.html)
  guide.
* Go: Ensure you have Go 1.24+ installed.

   _To verify Go is installed, you can run_
   ```bash
   go version
   ```

* You must have an Aurora DSQL cluster. For information about creating an Aurora DSQL cluster, see the
  [Getting started with Aurora DSQL](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/getting-started.html)
  guide.
* If connecting as a non-admin user, ensure the user is linked to an IAM role and is granted access to the `myschema`
  schema. See the
  [Using database roles with IAM roles](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/using-database-and-iam-roles.html)
  guide.

### Run the code

The example demonstrates the following operations:

- Opening a connection pool to an Aurora DSQL cluster
- Verifying connectivity
- Running concurrent queries across multiple goroutines

The example is designed to work with both admin and non-admin users:

- When run as an admin user, it uses the `public` schema
- When run as a non-admin user, it uses the `myschema` schema

**Note:** running the example will use actual resources in your AWS account and may incur charges.

Set environment variables for your cluster details:

```bash
# e.g. "admin"
export CLUSTER_USER="<your user>"

# e.g. "foo0bar1baz2quux3quuux4.dsql.us-east-1.on.aws"
export CLUSTER_ENDPOINT="<your endpoint>"
```

Run the tests:

```bash
go test ./test/... -v
```

Run a specific example test:

```bash
# Preferred example (concurrent pool queries)
go test ./test/ -run TestExamplePreferred -v

# Transaction example
go test ./test/transaction/... -v

# OCC retry example
go test ./test/occ_retry/... -v

# Connection string example
go test ./test/connection_string/... -v
```

The example contains comments explaining the code and the operations being performed.

## Additional resources

* [Amazon Aurora DSQL Documentation](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/what-is-aurora-dsql.html)
* [Amazon Aurora DSQL pgx Connector](https://github.com/awslabs/aurora-dsql-connectors/tree/main/go/pgx)
* [pgx Documentation](https://pkg.go.dev/github.com/jackc/pgx/v5)
* [AWS SDK for Go v2 Documentation](https://pkg.go.dev/github.com/aws/aws-sdk-go-v2)

---

Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.

SPDX-License-Identifier: Apache-2.0
