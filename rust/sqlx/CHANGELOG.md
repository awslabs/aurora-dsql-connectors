## Unreleased

Initial release of Aurora DSQL SQLx Connector for Rust

### Features
- `DsqlConnectOptions` wrapping `PgConnectOptions` with builder pattern via `derive_builder`
- Automatic IAM token generation (admin and regular user tokens)
- Reusable `AuthTokenGenerator` signer — built once per pool, reused across token refreshes
- Connection pooling with background token refresh at 80% of token duration (opt-in `pool` feature)
- Single connection support via `connection::connect()` / `connection::connect_with()`
- Region auto-detection from endpoint hostname or AWS SDK defaults
- Cluster ID shorthand expansion (e.g. `postgres://admin@<cluster_id>/postgres?region=us-east-1`)
- Support for AWS profiles
- SSL always enabled with `verify-full` mode
- Connection string parsing with configurable query parameters
- OCC retry helpers (`retry_on_occ`, `is_occ_error`) with SQLSTATE-based detection and exponential backoff (opt-in `occ` feature)
- `DsqlError` enum with `#[non_exhaustive]` and proper error source chaining via `thiserror`
- No default features — `occ` and `pool` are opt-in to minimize dependency footprint
