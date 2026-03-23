// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

using Amazon.AuroraDsql.Npgsql;
using Xunit;

namespace Amazon.AuroraDsql.Npgsql.Tests;

public class ConfigTests
{
    [Fact]
    public void Resolve_WithFullHostname_AppliesDefaults()
    {
        var config = new DsqlConfig { Host = "cluster.dsql.us-east-1.on.aws" };
        var resolved = config.ResolveInternal();

        Assert.Equal("cluster.dsql.us-east-1.on.aws", resolved.Host);
        Assert.Equal("us-east-1", resolved.Region);
        Assert.Equal("admin", resolved.User);
        Assert.Equal("postgres", resolved.Database);
        Assert.Equal(5432, resolved.Port);
        Assert.Null(resolved.TokenDurationSecs);
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
        var resolved = config.ResolveInternal();

        Assert.Equal("abcdefghijklmnopqrstuvwxyz.dsql.eu-west-1.on.aws", resolved.Host);
        Assert.Equal("eu-west-1", resolved.Region);
    }

    [Fact]
    public void Validate_ClusterIdWithoutRegion_ThrowsDsqlException()
    {
        // Suppress RegionResolver so the test doesn't depend on the host
        // machine's AWS config or env vars.
        var config = new DsqlConfig
        {
            Host = "abcdefghijklmnopqrstuvwxyz",
            RegionResolver = () => null
        };
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
        var config = new DsqlConfig();
        Assert.Throws<DsqlException>(() => config.Validate());
    }

    [Fact]
    public void Validate_NegativeOccMaxRetries_ThrowsDsqlException()
    {
        var config = new DsqlConfig
        {
            Host = "cluster.dsql.us-east-1.on.aws",
            OccMaxRetries = -1
        };
        var ex = Assert.Throws<DsqlException>(() => config.Validate());
        Assert.Contains("OccMaxRetries", ex.Message);
    }

    [Fact]
    public void Resolve_CustomUser_Preserved()
    {
        var config = new DsqlConfig
        {
            Host = "cluster.dsql.us-east-1.on.aws",
            User = "myuser"
        };
        var resolved = config.ResolveInternal();
        Assert.Equal("myuser", resolved.User);
    }

    [Fact]
    public void Resolve_TokenDurationSecs_Preserved()
    {
        var config = new DsqlConfig
        {
            Host = "cluster.dsql.us-east-1.on.aws",
            TokenDurationSecs = 450,
            OccMaxRetries = 5
        };
        var resolved = config.ResolveInternal();
        Assert.Equal(450, resolved.TokenDurationSecs);
        Assert.Equal(5, resolved.OccMaxRetries);
    }

    [Fact]
    public void Resolve_ApplicationName_SetCorrectly()
    {
        var config = new DsqlConfig { Host = "cluster.dsql.us-east-1.on.aws" };
        var resolved = config.ResolveInternal();
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

    [Fact]
    public void ParseConnectionString_EmptyProfile_Throws()
    {
        Assert.Throws<DsqlException>(() =>
            DsqlConfig.FromConnectionString(
                "postgres://admin@cluster.dsql.us-east-1.on.aws/postgres?profile="));
    }

    [Fact]
    public void ParseConnectionString_UnrecognizedParam_Throws()
    {
        var ex = Assert.Throws<DsqlException>(() =>
            DsqlConfig.FromConnectionString(
                "postgres://admin@cluster.dsql.us-east-1.on.aws/postgres?regin=us-west-2"));
        Assert.Contains("regin", ex.Message);
    }
}
