// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

using Amazon.AuroraDsql.Npgsql;
using Npgsql;
using Xunit;

namespace Amazon.AuroraDsql.Npgsql.IntegrationTests;

/// <summary>
/// Shared xUnit class fixture for integration tests.
/// Creates a DsqlDataSource once per test class and provides unique table names.
/// Tests fail when CLUSTER_ENDPOINT is not set.
/// </summary>
public class IntegrationTestFixture : IAsyncLifetime
{
    private const int DnsRetryCount = 3;
    private static readonly TimeSpan DnsRetryDelay = TimeSpan.FromSeconds(5);

    /// <summary>Shared data source for all tests in the class.</summary>
    public DsqlDataSource DataSource { get; private set; } = null!;

    /// <summary>Cluster endpoint from environment.</summary>
    public string Endpoint { get; private set; } = string.Empty;

    /// <summary>Cluster user from environment (default: "admin").</summary>
    public string User { get; private set; } = "admin";

    /// <summary>AWS region from environment.</summary>
    public string? Region { get; private set; }

    /// <summary>
    /// Generates a unique table name to avoid conflicts between test runs.
    /// </summary>
    public string GenerateTableName(string prefix = "integ")
    {
        var suffix = Guid.NewGuid().ToString("N")[..8];
        return $"{prefix}_{suffix}";
    }

    public async Task InitializeAsync()
    {
        Endpoint = Environment.GetEnvironmentVariable("CLUSTER_ENDPOINT")
            ?? throw new InvalidOperationException(
                "CLUSTER_ENDPOINT environment variable is required for integration tests.");

        User = Environment.GetEnvironmentVariable("CLUSTER_USER") ?? "admin";
        Region = Environment.GetEnvironmentVariable("REGION");

        var config = new DsqlConfig
        {
            Host = Endpoint,
            User = User,
        };

        if (!string.IsNullOrEmpty(Region))
            config.Region = Region;

        DataSource = await AuroraDsql.CreateDataSourceAsync(config);

        // Retry initial connection to handle DNS propagation delay
        // on freshly created clusters in CI.
        for (int attempt = 1; attempt <= DnsRetryCount; attempt++)
        {
            try
            {
                await using var conn = await DataSource.OpenConnectionAsync();
                await using var cmd = new NpgsqlCommand("SELECT 1", conn);
                await cmd.ExecuteScalarAsync();
                return;
            }
            catch (Exception) when (attempt < DnsRetryCount)
            {
                await Task.Delay(DnsRetryDelay);
            }
        }
    }

    public async Task DisposeAsync()
    {
        if (DataSource != null)
            await DataSource.DisposeAsync();
    }
}
