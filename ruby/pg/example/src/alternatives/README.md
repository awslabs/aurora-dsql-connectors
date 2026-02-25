# Alternative Examples

These examples show alternative approaches to connecting to Aurora DSQL when the standard connector doesn't meet your needs.

## When to Use Alternatives

Use the [preferred example](../example_preferred.rb) unless you have specific requirements:

| Alternative | Use When |
|-------------|----------|
| [manual_token](./manual_token/) | You need custom token generation logic, non-standard authentication flows, or want to understand the underlying mechanism |

## Running Examples

Each example has its own directory with source code:

```bash
cd /path/to/aurora-dsql-connectors/ruby/pg/example
export CLUSTER_ENDPOINT=your-cluster.dsql.us-east-1.on.aws
export CLUSTER_USER=admin
export REGION=us-east-1
ruby src/alternatives/manual_token/example.rb
```

## DSQL Best Practices

All examples follow DSQL best practices. See the [main README](../../../README.md#dsql-best-practices) for the complete list.
