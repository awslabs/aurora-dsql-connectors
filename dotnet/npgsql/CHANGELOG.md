<a id="dotnet/npgsql/v1.0.0"></a>
# Aurora DSQL Connector for .NET Npgsql v1.0.0 (dotnet/npgsql/v1.0.0)

Initial release of Aurora DSQL .NET Npgsql Connector

### Features
- Automatic IAM token generation (admin and regular users)
- Connection pooling via NpgsqlDataSource with max_lifetime enforcement
- Single connection support for simpler use cases
- Opt-in OCC retry with exponential backoff
- Flexible host configuration (full endpoint or cluster ID)
- Region auto-detection from endpoint hostname
- Support for AWS profiles and custom credentials providers
- SSL always enabled with verify-full mode and direct TLS negotiation
- Connection string parsing support
