## Unreleased

Initial release of Aurora DSQL SQLx Connector for Rust

### Features
- Automatic IAM token generation (fresh token per connection)
- Connection pooling via bb8 (opt-in with `pool` feature flag)
- Single connection support for simpler use cases
- Region auto-detection from endpoint hostname
- Support for AWS profiles
- SSL always enabled with `verify-full` mode
- Connection string parsing support
- OCC retry helpers with exponential backoff and jitter
