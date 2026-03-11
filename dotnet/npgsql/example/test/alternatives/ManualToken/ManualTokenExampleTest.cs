// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

using Amazon.AuroraDsql.Npgsql.Examples.Alternatives;
using Xunit;

namespace Amazon.AuroraDsql.Npgsql.Examples.Tests.Alternatives;

[Collection("ExampleTests")]
public class ManualTokenExampleTest
{
    [SkippableFact]
    public async Task RunExample()
    {
        var endpoint = Environment.GetEnvironmentVariable("CLUSTER_ENDPOINT");
        var user = Environment.GetEnvironmentVariable("CLUSTER_USER");
        var region = Environment.GetEnvironmentVariable("REGION");

        Skip.If(string.IsNullOrEmpty(endpoint) ||
            string.IsNullOrEmpty(user) ||
            string.IsNullOrEmpty(region),
            "Requires CLUSTER_ENDPOINT, CLUSTER_USER, and REGION environment variables");

        await ManualTokenExample.RunAsync(endpoint!);
    }
}
