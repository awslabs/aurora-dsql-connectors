// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

using Amazon.AuroraDsql.Npgsql;
using Xunit;

namespace Amazon.AuroraDsql.Npgsql.Tests;

public class ConfigTests
{
    /// <summary>
    /// Creates a DsqlConfig with RegionResolver suppressed so tests
    /// don't depend on the host machine's ~/.aws/config or env vars.
    /// </summary>
    private static DsqlConfig MakeConfig(string? host = null)
    {
        var config = new DsqlConfig { RegionResolver = () => null };
        if (host != null) config.Host = host;
        return config;
    }

    [Fact]
    public void Resolve_WithFullHostname_AppliesDefaults()
    {
        var config = MakeConfig("cluster.dsql.us-east-1.on.aws");
        var resolved = config.ResolveInternal();

        Assert.Equal("cluster.dsql.us-east-1.on.aws", resolved.Host);
        Assert.Equal("us-east-1", resolved.Region);
        Assert.Equal("admin", resolved.User);
        Assert.Equal("postgres", resolved.Database);
        Assert.Equal(5432, resolved.Port);
        Assert.Equal(10, resolved.MaxPoolSize);
        Assert.Equal(0, resolved.MinPoolSize);
        Assert.Equal(3300, resolved.ConnectionLifetime);
        Assert.Equal(600, resolved.ConnectionIdleLifetime);
        Assert.Null(resolved.OccMaxRetries);
    }

    [Fact]
    public void Resolve_WithClusterId_ExpandsHostname()
    {
        var config = MakeConfig("abcdefghijklmnopqrstuvwxyz");
        config.Region = "eu-west-1";
        var resolved = config.ResolveInternal();

        Assert.Equal("abcdefghijklmnopqrstuvwxyz.dsql.eu-west-1.on.aws", resolved.Host);
        Assert.Equal("eu-west-1", resolved.Region);
    }

    [Fact]
    public void Validate_ClusterIdWithoutRegion_ThrowsDsqlException()
    {
        var config = MakeConfig("abcdefghijklmnopqrstuvwxyz");
        var ex = Assert.Throws<DsqlException>(() => config.Validate());
        Assert.Contains("region", ex.Message, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void Resolve_ClusterIdWithRegionFromResolver_UsesIt()
    {
        var config = new DsqlConfig
        {
            Host = "abcdefghijklmnopqrstuvwxyz",
            RegionResolver = () => "ap-southeast-1"
        };
        var resolved = config.ResolveInternal();

        Assert.Equal("abcdefghijklmnopqrstuvwxyz.dsql.ap-southeast-1.on.aws", resolved.Host);
        Assert.Equal("ap-southeast-1", resolved.Region);
    }

    [Fact]
    public void Validate_MissingHost_ThrowsDsqlException()
    {
        var config = MakeConfig();
        Assert.Throws<DsqlException>(() => config.Validate());
    }

    [Fact]
    public void Validate_InvalidPort_ThrowsDsqlException()
    {
        var config = MakeConfig("cluster.dsql.us-east-1.on.aws");
        config.Port = 0;
        Assert.Throws<DsqlException>(() => config.Validate());
    }

    [Fact]
    public void Validate_MinPoolSizeExceedsMax_ThrowsDsqlException()
    {
        var config = MakeConfig("cluster.dsql.us-east-1.on.aws");
        config.MinPoolSize = 50;
        config.MaxPoolSize = 10;
        var ex = Assert.Throws<DsqlException>(() => config.Validate());
        Assert.Contains("MinPoolSize", ex.Message);
    }

    [Fact]
    public void Validate_MaxPoolSizeZero_ThrowsDsqlException()
    {
        var config = MakeConfig("cluster.dsql.us-east-1.on.aws");
        config.MaxPoolSize = 0;
        var ex = Assert.Throws<DsqlException>(() => config.Validate());
        Assert.Contains("MaxPoolSize", ex.Message);
    }

    [Fact]
    public void Validate_NegativeConnectionLifetime_ThrowsDsqlException()
    {
        var config = MakeConfig("cluster.dsql.us-east-1.on.aws");
        config.ConnectionLifetime = -1;
        var ex = Assert.Throws<DsqlException>(() => config.Validate());
        Assert.Contains("ConnectionLifetime", ex.Message);
    }

    [Fact]
    public void Validate_NegativeConnectionIdleLifetime_ThrowsDsqlException()
    {
        var config = MakeConfig("cluster.dsql.us-east-1.on.aws");
        config.ConnectionIdleLifetime = -1;
        var ex = Assert.Throws<DsqlException>(() => config.Validate());
        Assert.Contains("ConnectionIdleLifetime", ex.Message);
    }

    [Fact]
    public void Resolve_CustomUser_Preserved()
    {
        var config = MakeConfig("cluster.dsql.us-east-1.on.aws");
        config.User = "myuser";
        var resolved = config.ResolveInternal();
        Assert.Equal("myuser", resolved.User);
    }

    [Fact]
    public void Resolve_CustomPoolSettings_Preserved()
    {
        var config = MakeConfig("cluster.dsql.us-east-1.on.aws");
        config.MaxPoolSize = 50;
        config.MinPoolSize = 5;
        config.ConnectionLifetime = 1800;
        config.ConnectionIdleLifetime = 300;
        config.OccMaxRetries = 5;
        var resolved = config.ResolveInternal();
        Assert.Equal(50, resolved.MaxPoolSize);
        Assert.Equal(5, resolved.MinPoolSize);
        Assert.Equal(1800, resolved.ConnectionLifetime);
        Assert.Equal(300, resolved.ConnectionIdleLifetime);
        Assert.Equal(5, resolved.OccMaxRetries);
    }

    [Fact]
    public void Resolve_ApplicationName_SetCorrectly()
    {
        var config = MakeConfig("cluster.dsql.us-east-1.on.aws");
        var resolved = config.ResolveInternal();
        Assert.StartsWith("aurora-dsql-dotnet-npgsql/", resolved.ApplicationName);
    }

    [Fact]
    public void Resolve_OrmPrefix_PrependedToApplicationName()
    {
        var config = MakeConfig("cluster.dsql.us-east-1.on.aws");
        config.OrmPrefix = "efcore";
        var resolved = config.ResolveInternal();
        Assert.StartsWith("efcore:aurora-dsql-dotnet-npgsql/", resolved.ApplicationName);
    }

    // --- Connection String Parsing ---

    [Fact]
    public void ParseConnectionString_ValidUri_ExtractsDsqlParams()
    {
        var config = DsqlConfig.FromConnectionString(
            "postgres://myuser@cluster.dsql.us-east-1.on.aws/postgres?profile=dev");

        Assert.Equal("cluster.dsql.us-east-1.on.aws", config.Host);
        Assert.Equal("myuser", config.User);
        Assert.Equal("dev", config.Profile);
    }

    [Fact]
    public void ParseConnectionString_PostgresqlScheme_Works()
    {
        var config = DsqlConfig.FromConnectionString(
            "postgresql://admin@cluster.dsql.us-east-1.on.aws/postgres");
        Assert.Equal("cluster.dsql.us-east-1.on.aws", config.Host);
    }

    [Fact]
    public void ParseConnectionString_EmptyRegion_Throws()
    {
        Assert.Throws<DsqlException>(() =>
            DsqlConfig.FromConnectionString(
                "postgres://admin@cluster.dsql.us-east-1.on.aws/postgres?region="));
    }

    [Fact]
    public void ParseConnectionString_ExplicitRegion_Preserved()
    {
        var config = DsqlConfig.FromConnectionString(
            "postgres://admin@cluster.dsql.us-east-1.on.aws/postgres?region=us-west-2");
        Assert.Equal("us-west-2", config.Region);
    }
}
