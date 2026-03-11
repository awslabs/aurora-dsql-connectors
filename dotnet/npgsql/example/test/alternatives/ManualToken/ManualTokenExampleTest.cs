// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

using Amazon.AuroraDsql.Npgsql.Examples.Alternatives;
using Xunit;

namespace Amazon.AuroraDsql.Npgsql.Examples.Tests.Alternatives;

// Run all example tests sequentially to avoid OCC conflicts on concurrent DDL
[Collection("ExampleTests")]
public class ManualTokenExampleTest
{
    [Fact]
    public async Task RunExample()
    {
        var endpoint = Environment.GetEnvironmentVariable("CLUSTER_ENDPOINT");
        var user = Environment.GetEnvironmentVariable("CLUSTER_USER");
        var region = Environment.GetEnvironmentVariable("REGION");

        if (string.IsNullOrEmpty(endpoint) ||
            string.IsNullOrEmpty(user) ||
            string.IsNullOrEmpty(region))
            return; // Skip when required env vars are not set

        await ManualTokenExample.RunAsync(endpoint);
    }
}
