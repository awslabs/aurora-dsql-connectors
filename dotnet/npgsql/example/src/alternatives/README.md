# Alternative Examples

These examples show alternative approaches to connecting to Aurora DSQL when the standard connector doesn't meet your needs.

## When to Use Alternatives

Use the [preferred example](../ExamplePreferred.cs) unless you have specific requirements:

| Alternative | Use When |
|-------------|----------|
| [SingleConnection](./SingleConnection/) | You need a single unpooled connection for scripts or simple use cases |
| [ManualToken](./ManualToken/) | You need custom token generation logic, non-standard authentication flows, or want to understand the underlying mechanism |

## Running Examples

Each example has its own directory with source code and tests:

```bash
cd /path/to/aurora-dsql-connectors/dotnet/npgsql/example
export CLUSTER_ENDPOINT=your-cluster.dsql.us-east-1.on.aws
dotnet test --filter "ExamplePreferredTest"
```

For ManualToken, additional environment variables are required:

```bash
export CLUSTER_USER=admin
export REGION=us-east-1
dotnet test --filter "ManualTokenExampleTest"
```

