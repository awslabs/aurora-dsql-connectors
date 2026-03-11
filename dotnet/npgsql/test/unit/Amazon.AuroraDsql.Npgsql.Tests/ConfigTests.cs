// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

using Amazon.AuroraDsql.Npgsql;
using Xunit;

namespace Amazon.AuroraDsql.Npgsql.Tests;

public class ConfigTests : IDisposable
{
    private readonly Func<string?> _originalRegionResolver;

    public ConfigTests()
    {
        // Save and override region resolver so tests don't depend on ~/.aws/config
        _originalRegionResolver = DsqlConfig.RegionResolver;
        DsqlConfig.RegionResolver = () => null;
    }

    public void Dispose()
    {
        DsqlConfig.RegionResolver = _originalRegionResolver;
    }

    [Fact]
    public void Resolve_WithFullHostname_AppliesDefaults()
    {
        var config = new DsqlConfig { Host = "cluster.dsql.us-east-1.on.aws" };
        var resolved = config.Resolve();

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
        var config = new DsqlConfig
        {
            Host = "abcdefghijklmnopqrstuvwxyz",
            Region = "eu-west-1"
        };
        var resolved = config.Resolve();

        Assert.Equal("abcdefghijklmnopqrstuvwxyz.dsql.eu-west-1.on.aws", resolved.Host);
        Assert.Equal("eu-west-1", resolved.Region);
    }

    [Fact]
    public void Resolve_ClusterIdWithoutRegion_ThrowsDsqlException()
    {
        var config = new DsqlConfig { Host = "abcdefghijklmnopqrstuvwxyz" };
        var ex = Assert.Throws<DsqlException>(() => config.Resolve());
        Assert.Contains("region", ex.Message, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void Resolve_ClusterIdWithRegionFromResolver_UsesIt()
    {
        DsqlConfig.RegionResolver = () => "ap-southeast-1";
        var config = new DsqlConfig { Host = "abcdefghijklmnopqrstuvwxyz" };
        var resolved = config.Resolve();

        Assert.Equal("abcdefghijklmnopqrstuvwxyz.dsql.ap-southeast-1.on.aws", resolved.Host);
        Assert.Equal("ap-southeast-1", resolved.Region);
    }

    [Fact]
    public void Resolve_MissingHost_ThrowsDsqlException()
    {
        var config = new DsqlConfig();
        Assert.Throws<DsqlException>(() => config.Resolve());
    }

    [Fact]
    public void Resolve_InvalidPort_ThrowsDsqlException()
    {
        var config = new DsqlConfig { Host = "cluster.dsql.us-east-1.on.aws", Port = 0 };
        Assert.Throws<DsqlException>(() => config.Resolve());
    }

    [Fact]
    public void Resolve_MinPoolSizeExceedsMax_ThrowsDsqlException()
    {
        var config = new DsqlConfig
        {
            Host = "cluster.dsql.us-east-1.on.aws",
            MinPoolSize = 50,
            MaxPoolSize = 10
        };
        var ex = Assert.Throws<DsqlException>(() => config.Resolve());
        Assert.Contains("MinPoolSize", ex.Message);
    }

    [Fact]
    public void Resolve_CustomUser_Preserved()
    {
        var config = new DsqlConfig
        {
            Host = "cluster.dsql.us-east-1.on.aws",
            User = "myuser"
        };
        var resolved = config.Resolve();
        Assert.Equal("myuser", resolved.User);
    }

    [Fact]
    public void Resolve_CustomPoolSettings_Preserved()
    {
        var config = new DsqlConfig
        {
            Host = "cluster.dsql.us-east-1.on.aws",
            MaxPoolSize = 50,
            MinPoolSize = 5,
            ConnectionLifetime = 1800,
            ConnectionIdleLifetime = 300,
            OccMaxRetries = 5
        };
        var resolved = config.Resolve();
        Assert.Equal(50, resolved.MaxPoolSize);
        Assert.Equal(5, resolved.MinPoolSize);
        Assert.Equal(1800, resolved.ConnectionLifetime);
        Assert.Equal(300, resolved.ConnectionIdleLifetime);
        Assert.Equal(5, resolved.OccMaxRetries);
    }

    [Fact]
    public void Resolve_ApplicationName_SetCorrectly()
    {
        var config = new DsqlConfig { Host = "cluster.dsql.us-east-1.on.aws" };
        var resolved = config.Resolve();
        Assert.StartsWith("aurora-dsql-dotnet-npgsql/", resolved.ApplicationName);
    }

    [Fact]
    public void Resolve_OrmPrefix_PrependedToApplicationName()
    {
        var config = new DsqlConfig
        {
            Host = "cluster.dsql.us-east-1.on.aws",
            OrmPrefix = "efcore"
        };
        var resolved = config.Resolve();
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
