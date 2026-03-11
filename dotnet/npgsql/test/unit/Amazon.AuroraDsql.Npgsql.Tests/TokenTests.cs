// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

using Amazon.AuroraDsql.Npgsql;
using Xunit;

namespace Amazon.AuroraDsql.Npgsql.Tests;

public class TokenTests
{
    [Fact]
    public void GenerateToken_AdminUser_UsesAdminMethod()
    {
        Assert.True(Token.IsAdminUser("admin"));
    }

    [Fact]
    public void GenerateToken_NonAdminUser_UsesRegularMethod()
    {
        Assert.False(Token.IsAdminUser("myuser"));
    }

    [Fact]
    public void GenerateToken_EmptyUser_IsNotAdmin()
    {
        Assert.False(Token.IsAdminUser(""));
    }

    [Fact]
    public void ResolveCredentials_CustomProvider_ReturnedDirectly()
    {
        var customCreds = new Amazon.Runtime.BasicAWSCredentials("test-key", "test-secret");
        var config = new ResolvedConfig(
            Host: "cluster.dsql.us-east-1.on.aws", Region: "us-east-1",
            User: "admin", Database: "postgres", Port: 5432,
            Profile: null,
            CustomCredentialsProvider: customCreds,
            MaxPoolSize: 10, MinPoolSize: 0,
            ConnectionLifetime: 3300, ConnectionIdleLifetime: 600,
            OccMaxRetries: null, OrmPrefix: null,
            ApplicationName: "test", LoggerFactory: null,
            ConfigureConnectionString: null);

        var resolved = Token.ResolveCredentials(config);
        Assert.Same(customCreds, resolved);
    }
}
