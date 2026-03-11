// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

using Amazon.AuroraDsql.Npgsql;
using Npgsql;
using Xunit;

namespace Amazon.AuroraDsql.Npgsql.IntegrationTests;

/// <summary>
/// Shared xUnit class fixture for integration tests.
/// Creates a DsqlDataSource once per test class and provides unique table names.
/// Tests are skipped when CLUSTER_ENDPOINT is not set.
/// </summary>
public class IntegrationTestFixture : IAsyncLifetime
{
    private const int DnsRetryCount = 3;
    private static readonly TimeSpan DnsRetryDelay = TimeSpan.FromSeconds(5);

    /// <summary>True when a cluster endpoint is available and the data source is ready.</summary>
    public bool IsAvailable { get; private set; }

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
        Endpoint = Environment.GetEnvironmentVariable("CLUSTER_ENDPOINT") ?? string.Empty;
        if (string.IsNullOrEmpty(Endpoint))
        {
            IsAvailable = false;
            return;
        }

        User = Environment.GetEnvironmentVariable("CLUSTER_USER") ?? "admin";
        Region = Environment.GetEnvironmentVariable("REGION");

        var config = new DsqlConfig
        {
            Host = Endpoint,
            User = User,
        };

        if (!string.IsNullOrEmpty(Region))
            config.Region = Region;

        DataSource = AuroraDsql.CreateDataSource(config);

        // Retry initial connection to handle DNS propagation delay
        // on freshly created clusters in CI.
        for (int attempt = 1; attempt <= DnsRetryCount; attempt++)
        {
            try
            {
                await using var conn = await DataSource.OpenConnectionAsync();
                await using var cmd = new NpgsqlCommand("SELECT 1", conn);
                await cmd.ExecuteScalarAsync();
                IsAvailable = true;
                return;
            }
            catch (Exception) when (attempt < DnsRetryCount)
            {
                await Task.Delay(DnsRetryDelay);
            }
        }

        // Final attempt — let it throw if it fails
        await using (var conn = await DataSource.OpenConnectionAsync())
        {
            await using var cmd = new NpgsqlCommand("SELECT 1", conn);
            await cmd.ExecuteScalarAsync();
        }
        IsAvailable = true;
    }

    public async Task DisposeAsync()
    {
        if (DataSource != null)
            await DataSource.DisposeAsync();
    }
}
