# Manual Token Generation Example

This example demonstrates manual IAM token generation for Aurora DSQL connections using the `pg` gem directly.

## Required Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `CLUSTER_ENDPOINT` | Aurora DSQL cluster endpoint | Yes |
| `CLUSTER_USER` | Database user (`admin` or custom user) | Yes |
| `REGION` | AWS region (e.g., `us-east-1`) | Yes |

## Running the Example

```bash
export CLUSTER_ENDPOINT="your-cluster.dsql.us-east-1.on.aws"
export CLUSTER_USER="admin"
export REGION="us-east-1"

ruby example.rb
```

## When to Use Manual Token Generation

Use this approach when you need:

- **Custom token generation logic**: Implement custom caching, refresh strategies, or token lifecycle management.
- **Non-standard authentication flows**: Integrate with custom credential providers or assume roles with specific configurations.
- **Understanding the mechanism**: Learn how IAM authentication works under the hood before using higher-level abstractions.
- **Fine-grained control**: Customize token expiry, handle token generation errors differently, or implement retry logic.

## Key Differences from Preferred Approach

| Aspect | Manual Token (this example) | Preferred (`aurora-dsql-ruby-pg`) |
|--------|----------------------------|-----------------------------------|
| Token generation | Explicit in `create_connection` | Handled automatically by connector |
| Configuration | Build connection params manually | Use connector's configuration options |
| Maintenance | More code to maintain | Less boilerplate, maintained by AWS |
| Flexibility | Full control over token lifecycle | Opinionated but sufficient for most cases |

For most production applications, prefer the `aurora-dsql-ruby-pg` connector approach as it reduces boilerplate and follows AWS best practices. Use manual token generation only when you have specific requirements that the connector cannot satisfy.
