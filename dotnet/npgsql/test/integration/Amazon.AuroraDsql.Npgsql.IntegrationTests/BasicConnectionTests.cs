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
            await using (var conn = await _fixture.DataSource.OpenConnectionAsync())
            {
                await using var cmd = new NpgsqlCommand(
                    $"CREATE TABLE {table} (id UUID DEFAULT gen_random_uuid() PRIMARY KEY, value INT NOT NULL)",
                    conn);
                await cmd.ExecuteNonQueryAsync();
            }

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
            // DROP TABLE
            await using var conn = await _fixture.DataSource.OpenConnectionAsync();
            await using var drop = new NpgsqlCommand($"DROP TABLE IF EXISTS {table}", conn);
            await drop.ExecuteNonQueryAsync();
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
            await using (var conn = await _fixture.DataSource.OpenConnectionAsync())
            {
                await using var cmd = new NpgsqlCommand(
                    $"CREATE TABLE {table} (id UUID DEFAULT gen_random_uuid() PRIMARY KEY, name TEXT NOT NULL)",
                    conn);
                await cmd.ExecuteNonQueryAsync();
            }

            // Transactional INSERT
            await using (var conn = await _fixture.DataSource.OpenConnectionAsync())
            {
                await using var tx = await conn.BeginTransactionAsync();
                await using var cmd = new NpgsqlCommand(
                    $"INSERT INTO {table} (name) VALUES ($1)", conn, tx);
                cmd.Parameters.AddWithValue("test-item");
                await cmd.ExecuteNonQueryAsync();
                await tx.CommitAsync();
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
            await using var conn = await _fixture.DataSource.OpenConnectionAsync();
            await using var drop = new NpgsqlCommand($"DROP TABLE IF EXISTS {table}", conn);
            await drop.ExecuteNonQueryAsync();
        }
    }
}
