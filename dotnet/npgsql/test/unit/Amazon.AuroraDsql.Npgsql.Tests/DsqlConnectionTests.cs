// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

using Npgsql;
using Amazon.AuroraDsql.Npgsql;
using Xunit;

namespace Amazon.AuroraDsql.Npgsql.Tests;

public class DsqlConnectionTests
{
    private ResolvedConfig MakeConfig(
        string user = "admin",
        Action<NpgsqlConnectionStringBuilder>? configureConnectionString = null) =>
        new(
            Host: "cluster.dsql.us-east-1.on.aws", Region: "us-east-1",
            User: user, Database: "postgres", Port: 5432, Profile: null,
            CustomCredentialsProvider: null,
            MaxPoolSize: 10, MinPoolSize: 0,
            ConnectionLifetime: 3300, ConnectionIdleLifetime: 600,
            OccMaxRetries: null, OrmPrefix: null,
            ApplicationName: ConnectorVersion.ApplicationName,
            LoggerFactory: null,
            ConfigureConnectionString: configureConnectionString);

    [Fact]
    public void BuildConnectionString_PoolingDisabled()
    {
        var csb = DsqlConnection.BuildConnectionStringBuilder(MakeConfig());
        Assert.False(csb.Pooling);
    }

    [Fact]
    public void BuildConnectionString_SslDefaults()
    {
        var csb = DsqlConnection.BuildConnectionStringBuilder(MakeConfig());
        Assert.Equal(SslMode.VerifyFull, csb.SslMode);
        Assert.Equal(SslNegotiation.Direct, csb.SslNegotiation);
    }

    [Fact]
    public void BuildConnectionString_NoPasswordInConnectionString()
    {
        var csb = DsqlConnection.BuildConnectionStringBuilder(MakeConfig());
        Assert.True(string.IsNullOrEmpty(csb.Password));
    }

    [Fact]
    public void BuildConnectionString_ApplicationName()
    {
        var csb = DsqlConnection.BuildConnectionStringBuilder(MakeConfig());
        Assert.StartsWith("aurora-dsql-dotnet-npgsql/", csb.ApplicationName);
    }

    [Fact]
    public void BuildConnectionString_EnlistDisabled()
    {
        var csb = DsqlConnection.BuildConnectionStringBuilder(MakeConfig());
        Assert.False(csb.Enlist);
    }

    [Fact]
    public void BuildConnectionString_ConfigureConnectionStringApplied()
    {
        var csb = DsqlConnection.BuildConnectionStringBuilder(
            MakeConfig(configureConnectionString: b => b.CommandTimeout = 60));
        Assert.Equal(60, csb.CommandTimeout);
    }

    [Fact]
    public void BuildConnectionString_CallbackCannotOverrideSslMode()
    {
        var csb = DsqlConnection.BuildConnectionStringBuilder(
            MakeConfig(configureConnectionString: b => b.SslMode = SslMode.Disable));
        Assert.Equal(SslMode.VerifyFull, csb.SslMode);
    }

    [Fact]
    public void BuildConnectionString_CallbackCannotOverrideEnlist()
    {
        var csb = DsqlConnection.BuildConnectionStringBuilder(
            MakeConfig(configureConnectionString: b => b.Enlist = true));
        Assert.False(csb.Enlist);
    }
}
