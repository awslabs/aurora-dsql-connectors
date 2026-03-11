// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

using Amazon.AuroraDsql.Npgsql.Examples.Alternatives;
using Xunit;

namespace Amazon.AuroraDsql.Npgsql.Examples.Tests.Alternatives;

public class SingleConnectionExampleTest
{
    [Fact]
    public async Task RunExample()
    {
        var endpoint = Environment.GetEnvironmentVariable("CLUSTER_ENDPOINT");
        if (string.IsNullOrEmpty(endpoint))
            return; // Skip when no cluster available

        await SingleConnectionExample.RunAsync(endpoint);
    }
}
