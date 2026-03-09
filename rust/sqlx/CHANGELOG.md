## Unreleased

Initial release of Aurora DSQL SQLx Connector for Rust

### Features
- Automatic IAM token generation (fresh token per connection)
- Connection pooling via bb8 (opt-in with `pool` feature flag)
- Single connection support via `dsql_connect`
- Region auto-detection from endpoint hostname or AWS SDK defaults
- Cluster ID shorthand expansion (e.g. `postgres://admin@<cluster_id>/postgres?region=us-east-1`)
- Support for AWS profiles
- SSL always enabled with `verify-full` mode
- Connection string parsing with configurable query parameters
- Builder pattern for `DsqlConfig` and `OCCRetryConfig` via `derive_builder`
- Custom `PgConnectOptions` passthrough for driver-level settings
- OCC retry helpers (`retry_on_occ`, `with_retry`) with exponential backoff and jitter
- Strong types for domain values (`Host`, `Region`, `User`, `ClusterId`)
- `DsqlError` enum with `#[non_exhaustive]` and proper error source chaining via `thiserror`
