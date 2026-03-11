// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

using Amazon.AuroraDsql.Npgsql;
using Xunit;

namespace Amazon.AuroraDsql.Npgsql.Tests;

public class UtilTests
{
    // --- ParseRegion ---

    [Theory]
    [InlineData("abc12345678901234567890.dsql.us-east-1.on.aws", "us-east-1")]
    [InlineData("cluster.dsql.eu-west-1.on.aws", "eu-west-1")]
    [InlineData("cluster.dsql.ap-southeast-2.on.aws", "ap-southeast-2")]
    [InlineData("cluster.dsqlx.us-west-2.on.aws", "us-west-2")]
    public void ParseRegion_ValidHostnames_ReturnsRegion(string host, string expected)
    {
        Assert.Equal(expected, Util.ParseRegion(host));
    }

    [Theory]
    [InlineData("localhost")]
    [InlineData("example.com")]
    [InlineData("")]
    public void ParseRegion_InvalidHostnames_ReturnsNull(string host)
    {
        Assert.Null(Util.ParseRegion(host));
    }

    // --- IsClusterId ---

    [Theory]
    [InlineData("abcdefghijklmnopqrstuvwxyz")]
    [InlineData("12345678901234567890123456")]
    [InlineData("abc1234567890123456789012x")]
    public void IsClusterId_Valid_ReturnsTrue(string input)
    {
        Assert.True(Util.IsClusterId(input));
    }

    [Theory]
    [InlineData("abc.def.ghi")]
    [InlineData("short")]
    [InlineData("ABCDEFGHIJKLMNOPQRSTUVWXYZ")]
    [InlineData("abcdefghijklmnopqrstuvwxy")]
    [InlineData("abcdefghijklmnopqrstuvwxyz1")]
    [InlineData("")]
    public void IsClusterId_Invalid_ReturnsFalse(string input)
    {
        Assert.False(Util.IsClusterId(input));
    }

    [Fact]
    public void IsClusterId_Null_ReturnsFalse()
    {
        Assert.False(Util.IsClusterId(null));
    }

    // --- BuildHostname ---

    [Fact]
    public void BuildHostname_ReturnsCorrectFormat()
    {
        Assert.Equal(
            "clusterid12345678901234ab.dsql.us-east-1.on.aws",
            Util.BuildHostname("clusterid12345678901234ab", "us-east-1"));
    }
}
