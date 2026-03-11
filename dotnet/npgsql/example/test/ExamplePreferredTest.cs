// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

using Amazon.AuroraDsql.Npgsql.Examples;
using Xunit;

namespace Amazon.AuroraDsql.Npgsql.Examples.Tests;

[Collection("ExampleTests")]
public class ExamplePreferredTest
{
    [SkippableFact]
    public async Task RunExample()
    {
        var endpoint = Environment.GetEnvironmentVariable("CLUSTER_ENDPOINT");
        Skip.If(string.IsNullOrEmpty(endpoint), "Requires CLUSTER_ENDPOINT environment variable");

        await ExamplePreferred.RunAsync(endpoint!);
    }
}
