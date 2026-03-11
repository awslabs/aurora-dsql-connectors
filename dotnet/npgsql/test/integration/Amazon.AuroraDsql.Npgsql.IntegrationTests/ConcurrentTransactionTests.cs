// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

using Amazon.AuroraDsql.Npgsql;
using Npgsql;
using Xunit;

namespace Amazon.AuroraDsql.Npgsql.IntegrationTests;

public class ConcurrentTransactionTests : IClassFixture<IntegrationTestFixture>
{
    private readonly IntegrationTestFixture _fixture;

    public ConcurrentTransactionTests(IntegrationTestFixture fixture) => _fixture = fixture;

    [Fact]
    public async Task ConcurrentIncrements_BothSucceedViaOccRetry()
    {
        if (!_fixture.IsAvailable) return;

        var table = _fixture.GenerateTableName("conc");
        const int maxRetries = 5;

        try
        {
            // Create counter table
            await OccRetry.ExecWithRetryAsync(_fixture.DataSource,
                $"CREATE TABLE {table} (id UUID DEFAULT gen_random_uuid() PRIMARY KEY, counter INT NOT NULL DEFAULT 0)");

            // Insert a row with counter = 0
            string rowId;
            await using (var conn = await _fixture.DataSource.OpenConnectionAsync())
            {
                await using var cmd = new NpgsqlCommand(
                    $"INSERT INTO {table} (counter) VALUES (0) RETURNING id", conn);
                rowId = ((Guid)(await cmd.ExecuteScalarAsync())!).ToString();
            }

            // Barrier to synchronize both tasks: ensures both read before either commits
            var barrier = new ManualResetEventSlim(false);
            var readCount = 0;

            var results = new Exception?[2];

            var tasks = Enumerable.Range(0, 2).Select(i => Task.Run(async () =>
            {
                try
                {
                    await OccRetry.WithRetryAsync(
                        _fixture.DataSource,
                        maxRetries: maxRetries,
                        async conn =>
                        {
                            // Read current counter value
                            await using var readCmd = new NpgsqlCommand(
                                $"SELECT counter FROM {table} WHERE id = $1::uuid", conn);
                            readCmd.Parameters.AddWithValue(rowId);
                            var current = (int)(await readCmd.ExecuteScalarAsync())!;

                            // Synchronize: wait until both tasks have read
                            if (Interlocked.Increment(ref readCount) >= 2)
                            {
                                barrier.Set();
                            }
                            else
                            {
                                barrier.Wait(TimeSpan.FromSeconds(10));
                            }

                            // Increment
                            await using var writeCmd = new NpgsqlCommand(
                                $"UPDATE {table} SET counter = $1 WHERE id = $2::uuid", conn);
                            writeCmd.Parameters.AddWithValue(current + 1);
                            writeCmd.Parameters.AddWithValue(rowId);
                            await writeCmd.ExecuteNonQueryAsync();
                        });
                }
                catch (Exception ex)
                {
                    results[i] = ex;
                }
            })).ToArray();

            await Task.WhenAll(tasks);

            // Both tasks should succeed (one via OCC retry)
            for (int i = 0; i < 2; i++)
            {
                Assert.Null(results[i]);
            }

            // Verify final counter value is 2 (both increments applied)
            await using (var conn = await _fixture.DataSource.OpenConnectionAsync())
            {
                await using var cmd = new NpgsqlCommand(
                    $"SELECT counter FROM {table} WHERE id = $1::uuid", conn);
                cmd.Parameters.AddWithValue(rowId);
                var finalValue = (int)(await cmd.ExecuteScalarAsync())!;
                Assert.Equal(2, finalValue);
            }
        }
        finally
        {
            await OccRetry.ExecWithRetryAsync(_fixture.DataSource,
                $"DROP TABLE IF EXISTS {table}");
        }
    }
}
