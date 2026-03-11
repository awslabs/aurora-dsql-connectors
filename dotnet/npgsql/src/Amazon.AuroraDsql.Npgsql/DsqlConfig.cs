// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

using Amazon.Runtime;
using Microsoft.Extensions.Logging;

namespace Amazon.AuroraDsql.Npgsql;

/// <summary>
/// Configuration for Aurora DSQL connections. Call <see cref="Validate"/> to check
/// configuration eagerly before creating a data source or connection.
/// </summary>
public class DsqlConfig
{
    /// <summary>Full DSQL endpoint or bare 26-char cluster ID.</summary>
    public string? Host { get; set; }

    /// <summary>AWS region. Optional when host is a full endpoint (parsed automatically).</summary>
    public string? Region { get; set; }

    /// <summary>Database user. Default: "admin".</summary>
    public string User { get; set; } = "admin";

    /// <summary>Database name. Default: "postgres".</summary>
    public string Database { get; set; } = "postgres";

    /// <summary>PostgreSQL port. Default: 5432.</summary>
    public int Port { get; set; } = 5432;

    /// <summary>AWS profile name for credential resolution.</summary>
    public string? Profile { get; set; }

    /// <summary>Explicit AWS credentials for cross-account or assume-role scenarios.</summary>
    public AWSCredentials? CustomCredentialsProvider { get; set; }

    /// <summary>Maximum pool size. Default: 10.</summary>
    public int MaxPoolSize { get; set; } = 10;

    /// <summary>Minimum pool size. Default: 0.</summary>
    public int MinPoolSize { get; set; }

    /// <summary>Max connection lifetime in seconds. Default: 3300 (55 min).</summary>
    public int ConnectionLifetime { get; set; } = 3300;

    /// <summary>Max idle time in seconds. Default: 600 (10 min).</summary>
    public int ConnectionIdleLifetime { get; set; } = 600;

    /// <summary>Enable OCC retry on ExecuteAsync when set to a positive integer. Default: null (disabled).</summary>
    public int? OccMaxRetries { get; set; }

    /// <summary>ORM prefix prepended to application_name (e.g., "efcore").</summary>
    public string? OrmPrefix { get; set; }

    /// <summary>Logger factory for retry warnings and diagnostics.</summary>
    public ILoggerFactory? LoggerFactory { get; set; }

    /// <summary>
    /// Region resolution strategy. Override in tests to control environment behavior.
    /// Default: reads from FallbackRegionFactory, then AWS_DEFAULT_REGION env var.
    /// </summary>
    internal Func<string?> RegionResolver { get; set; } = DefaultResolveRegionFromEnvironment;

    /// <summary>
    /// Validates the configuration and throws <see cref="DsqlException"/> if invalid.
    /// Call this to check configuration eagerly before creating a data source or connection.
    /// </summary>
    public void Validate() => ResolveInternal();

    /// <summary>
    /// Validates the configuration, applies defaults, and returns an immutable resolved config.
    /// </summary>
    internal ResolvedConfig ResolveInternal()
    {
        if (string.IsNullOrWhiteSpace(Host))
            throw new DsqlException("Host is required. Provide a full DSQL endpoint or a 26-character cluster ID.");

        if (Port < 1 || Port > 65535)
            throw new DsqlException($"Port must be between 1 and 65535, got {Port}.");

        if (MinPoolSize > MaxPoolSize)
            throw new DsqlException($"MinPoolSize ({MinPoolSize}) must not exceed MaxPoolSize ({MaxPoolSize}).");

        var host = Host;
        string? region = Region;

        if (Util.IsClusterId(host))
        {
            region ??= RegionResolver();
            if (string.IsNullOrWhiteSpace(region))
                throw new DsqlException(
                    "Region is required when Host is a cluster ID. " +
                    "Set Region in config, or set the AWS_REGION / AWS_DEFAULT_REGION environment variable.");
            host = Util.BuildHostname(host, region);
        }
        else
        {
            region ??= Util.ParseRegion(host);
            region ??= RegionResolver();
            if (string.IsNullOrWhiteSpace(region))
                throw new DsqlException(
                    "Could not determine AWS region. " +
                    "Provide a standard DSQL hostname, set Region in config, or set AWS_REGION.");
        }

        return new ResolvedConfig(
            Host: host,
            Region: region,
            User: User,
            Database: Database,
            Port: Port,
            Profile: Profile,
            CustomCredentialsProvider: CustomCredentialsProvider,
            MaxPoolSize: MaxPoolSize,
            MinPoolSize: MinPoolSize,
            ConnectionLifetime: ConnectionLifetime,
            ConnectionIdleLifetime: ConnectionIdleLifetime,
            OccMaxRetries: OccMaxRetries,
            OrmPrefix: OrmPrefix,
            ApplicationName: ConnectorVersion.BuildApplicationName(OrmPrefix),
            LoggerFactory: LoggerFactory);
    }

    /// <summary>
    /// Parses a postgres:// or postgresql:// connection string into a DsqlConfig.
    /// DSQL-specific params (region, profile) are extracted and stripped.
    /// </summary>
    public static DsqlConfig FromConnectionString(string connectionString)
    {
        if (string.IsNullOrWhiteSpace(connectionString))
            throw new DsqlException("Connection string must not be empty.");

        // Normalize scheme
        var uri = connectionString;
        if (uri.StartsWith("postgresql://", StringComparison.OrdinalIgnoreCase))
            uri = "postgres://" + uri["postgresql://".Length..];

        if (!uri.StartsWith("postgres://", StringComparison.OrdinalIgnoreCase))
            throw new DsqlException("Connection string must start with postgres:// or postgresql://.");

        var parsed = new Uri(uri);
        var query = System.Web.HttpUtility.ParseQueryString(parsed.Query);

        var config = new DsqlConfig
        {
            Host = parsed.Host,
            Database = parsed.AbsolutePath.TrimStart('/') is { Length: > 0 } db ? db : "postgres",
            Port = parsed.Port > 0 ? parsed.Port : 5432,
        };

        // Extract user from URI
        if (!string.IsNullOrEmpty(parsed.UserInfo))
        {
            var userInfo = parsed.UserInfo;
            var colonIdx = userInfo.IndexOf(':');
            config.User = colonIdx >= 0 ? userInfo[..colonIdx] : userInfo;
        }

        // Extract and strip DSQL-specific params
        var region = query.Get("region");
        if (region != null)
        {
            if (string.IsNullOrWhiteSpace(region))
                throw new DsqlException("Connection string parameter 'region' must not be empty.");
            config.Region = region;
        }

        var profile = query.Get("profile");
        if (profile != null)
            config.Profile = profile;

        return config;
    }

    private static string? DefaultResolveRegionFromEnvironment()
    {
        // FallbackRegionFactory checks: AWS_REGION env var → profile config → EC2/ECS metadata.
        // It does NOT check AWS_DEFAULT_REGION, so we fall back to that manually.
        try
        {
            var region = Amazon.Runtime.FallbackRegionFactory.GetRegionEndpoint();
            if (region != null)
                return region.SystemName;
        }
        catch
        {
            // FallbackRegionFactory may throw if no region configured
        }

        return Environment.GetEnvironmentVariable("AWS_DEFAULT_REGION");
    }
}

/// <summary>
/// Immutable resolved configuration with all defaults applied.
/// </summary>
internal sealed record ResolvedConfig(
    string Host,
    string Region,
    string User,
    string Database,
    int Port,
    string? Profile,
    AWSCredentials? CustomCredentialsProvider,
    int MaxPoolSize,
    int MinPoolSize,
    int ConnectionLifetime,
    int ConnectionIdleLifetime,
    int? OccMaxRetries,
    string? OrmPrefix,
    string ApplicationName,
    ILoggerFactory? LoggerFactory);
