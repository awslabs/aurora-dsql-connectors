// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

using Amazon.AuroraDsql.Npgsql;
using Npgsql;
using Xunit;

namespace Amazon.AuroraDsql.Npgsql.IntegrationTests;

public class BasicConnectionTests : IClassFixture<IntegrationTestFixture>
{
    private readonly IntegrationTestFixture _fixture;

    public BasicConnectionTests(IntegrationTestFixture fixture) => _fixture = fixture;

    [Fact]
    public async Task SelectOne()
    {
        if (!_fixture.IsAvailable) return;

        await using var conn = await _fixture.DataSource.OpenConnectionAsync();
        await using var cmd = new NpgsqlCommand("SELECT 1", conn);
        var result = await cmd.ExecuteScalarAsync();

        Assert.Equal(1, result);
    }

    [Fact]
    public async Task CreateInsertSelectDelete()
    {
        if (!_fixture.IsAvailable) return;

        var table = _fixture.GenerateTableName("basic");

        try
        {
            // CREATE
            await OccRetry.ExecWithRetryAsync(_fixture.DataSource,
                $"CREATE TABLE {table} (id UUID DEFAULT gen_random_uuid() PRIMARY KEY, value INT NOT NULL)");

            // INSERT
            await using (var conn = await _fixture.DataSource.OpenConnectionAsync())
            {
                await using var cmd = new NpgsqlCommand(
                    $"INSERT INTO {table} (value) VALUES ($1)", conn);
                cmd.Parameters.AddWithValue(42);
                await cmd.ExecuteNonQueryAsync();
            }

            // SELECT
            await using (var conn = await _fixture.DataSource.OpenConnectionAsync())
            {
                await using var cmd = new NpgsqlCommand(
                    $"SELECT value FROM {table} WHERE value = $1", conn);
                cmd.Parameters.AddWithValue(42);
                var result = await cmd.ExecuteScalarAsync();
                Assert.Equal(42, result);
            }

            // DELETE
            await using (var conn = await _fixture.DataSource.OpenConnectionAsync())
            {
                await using var cmd = new NpgsqlCommand(
                    $"DELETE FROM {table} WHERE value = $1", conn);
                cmd.Parameters.AddWithValue(42);
                var deleted = await cmd.ExecuteNonQueryAsync();
                Assert.Equal(1, deleted);
            }
        }
        finally
        {
            await OccRetry.ExecWithRetryAsync(_fixture.DataSource,
                $"DROP TABLE IF EXISTS {table}");
        }
    }

    [Fact]
    public async Task TransactionalWrite()
    {
        if (!_fixture.IsAvailable) return;

        var table = _fixture.GenerateTableName("txn");

        try
        {
            // Create the table
            await OccRetry.ExecWithRetryAsync(_fixture.DataSource,
                $"CREATE TABLE {table} (id UUID DEFAULT gen_random_uuid() PRIMARY KEY, name TEXT NOT NULL)");

            // Transactional INSERT using raw SQL BEGIN/COMMIT (DSQL uses fixed Repeatable Read isolation)
            await using (var conn = await _fixture.DataSource.OpenConnectionAsync())
            {
                await new NpgsqlCommand("BEGIN", conn).ExecuteNonQueryAsync();
                await using var cmd = new NpgsqlCommand(
                    $"INSERT INTO {table} (name) VALUES ($1)", conn);
                cmd.Parameters.AddWithValue("test-item");
                await cmd.ExecuteNonQueryAsync();
                await new NpgsqlCommand("COMMIT", conn).ExecuteNonQueryAsync();
            }

            // Verify the row was committed
            await using (var conn = await _fixture.DataSource.OpenConnectionAsync())
            {
                await using var cmd = new NpgsqlCommand(
                    $"SELECT COUNT(*) FROM {table} WHERE name = $1", conn);
                cmd.Parameters.AddWithValue("test-item");
                var count = (long)(await cmd.ExecuteScalarAsync())!;
                Assert.Equal(1, count);
            }
        }
        finally
        {
            await OccRetry.ExecWithRetryAsync(_fixture.DataSource,
                $"DROP TABLE IF EXISTS {table}");
        }
    }
}
