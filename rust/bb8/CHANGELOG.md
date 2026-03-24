## Unreleased

Initial release of Aurora DSQL bb8 Connection Pool for Rust

### Features
- `DsqlConnectionManager` implementing `bb8::ManageConnection` for Aurora DSQL
- Fresh IAM auth token generation per connection (no background refresh needed)
- Connection health checks via `is_valid` (ping)
- Built on top of the Aurora DSQL SQLx Connector (`DsqlConnectOptions`)
