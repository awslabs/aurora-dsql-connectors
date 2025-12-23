# Alternative Examples

The recommended approach is `example_preferred.js` in the parent directory, which uses postgres.js with the Aurora DSQL Node.js Connector.

## Why Connection Pooling with the Connector?

Aurora DSQL has specific connection characteristics:
- **60-minute max connection lifetime** - connections are terminated after 1 hour
- **15-minute token expiry** - IAM auth tokens must be refreshed
- **Optimized for concurrency** - more concurrent connections with smaller batches yields better throughput

The connector handles token generation automatically.

## Alternatives

### `no_connection_pool/`
Examples without pooling:
- `example_with_no_connection_pool.js` - Single connection with connector
