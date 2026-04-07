# Alternative Examples

The recommended approach is `example_preferred.rs` in the parent directory, which uses a connection pool with background token refresh and automatic OCC retry via the Aurora DSQL SQLx Connector.

## Why Connection Pooling with the Connector?

Aurora DSQL has specific connection characteristics:
- **60-minute max connection lifetime** — connections are terminated after 1 hour
- **IAM auth token expiry** — tokens can be valid for up to 7 days, but a 15-minute default is recommended for security best practices
- **Optimistic concurrency control (OCC)** — transactions may conflict and require retry
- **Optimized for concurrency** — more concurrent connections with smaller batches yields better throughput

The connector pool helper (`aurora_dsql_sqlx_connector::pool::connect`) with `OCCRetryExt` trait handles this automatically:
- Refreshes IAM tokens in the background before they expire
- Automatically retries transactions on OCC conflicts
- Works with sqlx's standard `PgPool` for connection lifecycle management

## Alternatives

### `no_connection_pool/`
Single connection without pooling or automatic OCC retry:
- `example_no_connection_pool.rs` — Direct connection using `connection::connect()`
- Best for simple scripts or cases where pooling overhead isn't needed
- Does not include automatic OCC retry (use `retry_on_occ` manually if needed)
