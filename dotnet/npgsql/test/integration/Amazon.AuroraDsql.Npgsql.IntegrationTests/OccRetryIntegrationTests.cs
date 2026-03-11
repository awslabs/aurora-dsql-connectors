// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

using Amazon.AuroraDsql.Npgsql;
using Npgsql;
using Xunit;

namespace Amazon.AuroraDsql.Npgsql.IntegrationTests;

public class OccRetryIntegrationTests : IClassFixture<IntegrationTestFixture>
{
    private readonly IntegrationTestFixture _fixture;

    public OccRetryIntegrationTests(IntegrationTestFixture fixture) => _fixture = fixture;

    [Fact]
    public async Task WithRetryAsync_NonConflictingWrite()
    {
        if (!_fixture.IsAvailable) return;

        var table = _fixture.GenerateTableName("occ_wr");

        try
        {
            // Create table
            await using (var conn = await _fixture.DataSource.OpenConnectionAsync())
            {
                await using var cmd = new NpgsqlCommand(
                    $"CREATE TABLE {table} (id UUID DEFAULT gen_random_uuid() PRIMARY KEY, value INT NOT NULL)",
                    conn);
                await cmd.ExecuteNonQueryAsync();
            }

            // Insert with OCC retry (no conflict expected, should succeed on first attempt)
            await OccRetry.WithRetryAsync(
                _fixture.DataSource,
                maxRetries: 3,
                async (conn, tx) =>
                {
                    await using var cmd = new NpgsqlCommand(
                        $"INSERT INTO {table} (value) VALUES ($1)", conn, tx);
                    cmd.Parameters.AddWithValue(42);
                    await cmd.ExecuteNonQueryAsync();
                });

            // Verify
            await using (var conn = await _fixture.DataSource.OpenConnectionAsync())
            {
                await using var cmd = new NpgsqlCommand(
                    $"SELECT COUNT(*) FROM {table} WHERE value = $1", conn);
                cmd.Parameters.AddWithValue(42);
                var count = (long)(await cmd.ExecuteScalarAsync())!;
                Assert.True(count >= 1, $"Expected at least 1 row with value=42, got {count}");
            }
        }
        finally
        {
            await using var conn = await _fixture.DataSource.OpenConnectionAsync();
            await using var drop = new NpgsqlCommand($"DROP TABLE IF EXISTS {table}", conn);
            await drop.ExecuteNonQueryAsync();
        }
    }

    [Fact]
    public async Task ExecWithRetryAsync_DDL()
    {
        if (!_fixture.IsAvailable) return;

        var table = _fixture.GenerateTableName("occ_ddl");

        try
        {
            // Execute DDL with OCC retry
            await OccRetry.ExecWithRetryAsync(
                _fixture.DataSource,
                $"CREATE TABLE IF NOT EXISTS {table} (id UUID DEFAULT gen_random_uuid() PRIMARY KEY, value INT NOT NULL)");

            // Verify the table exists by inserting and selecting
            await using (var conn = await _fixture.DataSource.OpenConnectionAsync())
            {
                await using var cmd = new NpgsqlCommand(
                    $"INSERT INTO {table} (value) VALUES ($1)", conn);
                cmd.Parameters.AddWithValue(1);
                await cmd.ExecuteNonQueryAsync();
            }

            await using (var conn = await _fixture.DataSource.OpenConnectionAsync())
            {
                await using var cmd = new NpgsqlCommand(
                    $"SELECT COUNT(*) FROM {table}", conn);
                var count = (long)(await cmd.ExecuteScalarAsync())!;
                Assert.True(count >= 1, $"Expected at least 1 row, got {count}");
            }
        }
        finally
        {
            await using var conn = await _fixture.DataSource.OpenConnectionAsync();
            await using var drop = new NpgsqlCommand($"DROP TABLE IF EXISTS {table}", conn);
            await drop.ExecuteNonQueryAsync();
        }
    }
}
