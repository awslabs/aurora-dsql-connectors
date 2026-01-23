# Alternative Examples

The recommended approach is `example_preferred.js` in the parent directory, which uses postgres.js with the Aurora DSQL Node.js Connector.

## Why Connection Pooling with the Connector?

Aurora DSQL has specific connection characteristics:
- **60-minute max connection lifetime** - connections are terminated after 1 hour
- **15-minute token expiry** - IAM auth tokens must be refreshed
- **Optimized for concurrency** - more concurrent connections with smaller batches yields better throughput

The connector handles token generation automatically.

## ⚠️ Important

- Running this code might result in charges to your AWS account.
- We recommend that you grant your code least privilege. At most, grant only the
  minimum permissions required to perform the task. For more information, see
  [Grant least privilege](https://docs.aws.amazon.com/IAM/latest/UserGuide/best-practices.html#grant-least-privilege).
- This code is not tested in every AWS Region. For more information, see
  [AWS Regional Services](https://aws.amazon.com/about-aws/global-infrastructure/regional-product-services).

## Alternatives

### `no_connection_pool/`
Examples without pooling:
- `example_with_no_connection_pool.js` - Single connection with connector

### `websocket/`
Aurora DSQL Query Editor React Sample
- Demonstrate the required webpack configuration for operating a WebSocket connector in the browser. 
