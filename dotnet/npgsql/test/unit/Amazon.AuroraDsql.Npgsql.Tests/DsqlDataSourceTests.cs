// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

using Npgsql;
using Amazon.AuroraDsql.Npgsql;
using Xunit;

namespace Amazon.AuroraDsql.Npgsql.Tests;

public class DsqlDataSourceTests
{
    private ResolvedConfig MakeConfig(
        string host = "cluster.dsql.us-east-1.on.aws",
        string region = "us-east-1",
        string user = "admin",
        int maxPoolSize = 10,
        int minPoolSize = 0,
        int connectionLifetime = 3300,
        int connectionIdleLifetime = 600,
        string? ormPrefix = null,
        Action<NpgsqlConnectionStringBuilder>? configureConnectionString = null) =>
        new(
            Host: host, Region: region, User: user,
            Database: "postgres", Port: 5432, Profile: null,
            CustomCredentialsProvider: null,
            MaxPoolSize: maxPoolSize, MinPoolSize: minPoolSize,
            ConnectionLifetime: connectionLifetime,
            ConnectionIdleLifetime: connectionIdleLifetime,
            OccMaxRetries: null, OrmPrefix: ormPrefix,
            ApplicationName: ConnectorVersion.BuildApplicationName(ormPrefix),
            LoggerFactory: null,
            ConfigureConnectionString: configureConnectionString);

    [Fact]
    public void BuildConnectionString_SslDefaults()
    {
        var csb = DsqlDataSource.BuildConnectionStringBuilder(MakeConfig());
        Assert.Equal(SslMode.VerifyFull, csb.SslMode);
        Assert.Equal(SslNegotiation.Direct, csb.SslNegotiation);
    }

    [Fact]
    public void BuildConnectionString_PoolSettings()
    {
        var csb = DsqlDataSource.BuildConnectionStringBuilder(MakeConfig(
            maxPoolSize: 50, minPoolSize: 5, connectionLifetime: 1800, connectionIdleLifetime: 300));
        Assert.Equal(50, csb.MaxPoolSize);
        Assert.Equal(5, csb.MinPoolSize);
        Assert.Equal(1800, csb.ConnectionLifetime);
        Assert.Equal(300, csb.ConnectionIdleLifetime);
    }

    [Fact]
    public void BuildConnectionString_ApplicationName()
    {
        var csb = DsqlDataSource.BuildConnectionStringBuilder(MakeConfig());
        Assert.StartsWith("aurora-dsql-dotnet-npgsql/", csb.ApplicationName);
    }

    [Fact]
    public void BuildConnectionString_OrmPrefix()
    {
        var csb = DsqlDataSource.BuildConnectionStringBuilder(MakeConfig(ormPrefix: "efcore"));
        Assert.StartsWith("efcore:aurora-dsql-dotnet-npgsql/", csb.ApplicationName);
    }

    [Fact]
    public void BuildConnectionString_EnlistDisabled()
    {
        var csb = DsqlDataSource.BuildConnectionStringBuilder(MakeConfig());
        Assert.False(csb.Enlist);
    }

    [Fact]
    public void BuildConnectionString_NoResetOnClose()
    {
        var csb = DsqlDataSource.BuildConnectionStringBuilder(MakeConfig());
        Assert.True(csb.NoResetOnClose);
    }

    [Fact]
    public void BuildConnectionString_HostAndPort()
    {
        var csb = DsqlDataSource.BuildConnectionStringBuilder(MakeConfig());
        Assert.Equal("cluster.dsql.us-east-1.on.aws", csb.Host);
        Assert.Equal(5432, csb.Port);
        Assert.Equal("postgres", csb.Database);
        Assert.Equal("admin", csb.Username);
    }
}
