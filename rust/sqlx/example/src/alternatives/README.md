# Alternative Examples

The recommended approach is `example_preferred.rs` in the parent directory, which uses `DsqlPool` with the Aurora DSQL SQLx Connector.

## Why Connection Pooling with the Connector?

Aurora DSQL has specific connection characteristics:
- **60-minute max connection lifetime** — connections are terminated after 1 hour
- **IAM auth token expiry** — tokens can be valid for up to 7 days, but a 15-minute default is recommended for security best practices
- **Optimized for concurrency** — more concurrent connections with smaller batches yields better throughput

The connector + pool combination handles this automatically:
- Generates fresh IAM tokens per connection
- Recycles connections before the 60-minute limit (default: 55 minutes)
- Reuses warmed connections for better performance

## Alternatives

### `no_connection_pool/`
Single connection without pooling:
- `example_no_connection_pool.rs` - Direct connection using `dsql_connect`
