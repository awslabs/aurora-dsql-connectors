# Alternative Examples

The recommended approach is `example_preferred.ts` in the parent directory, which uses AuroraDSQLPool with the Aurora DSQL Node.js Connector.

## Why Connection Pooling with the Connector?

Aurora DSQL has specific connection characteristics:
- **60-minute max connection lifetime** - connections are terminated after 1 hour
- **15-minute token expiry** - IAM auth tokens must be refreshed
- **Optimized for concurrency** - more concurrent connections with smaller batches yields better throughput

The connector + pool combination handles this automatically:
- Generates fresh IAM tokens per connection
- Recycles connections before the 60-minute limit
- Reuses warmed connections for better performance

## ⚠️ Important

- Running this code might result in charges to your AWS account.
- We recommend that you grant your code least privilege. At most, grant only the
  minimum permissions required to perform the task. For more information, see
  [Grant least privilege](https://docs.aws.amazon.com/IAM/latest/UserGuide/best-practices.html#grant-least-privilege).
- This code is not tested in every AWS Region. For more information, see
  [AWS Regional Services](https://aws.amazon.com/about-aws/global-infrastructure/regional-product-services).

## Alternatives

### `pool/`
Other pool configurations:
- `example_with_nonconcurrent_connection_pool.ts` - Sequential pool usage

### `no_connection_pool/`
Examples without pooling:
- `example_with_no_connection_pool.ts` - Single connection with connector
