// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

using Microsoft.Extensions.Logging;
using Npgsql;

namespace Amazon.AuroraDsql.Npgsql;

/// <summary>
/// OCC (Optimistic Concurrency Control) error detection and retry logic for Aurora DSQL.
/// </summary>
public static class OccRetry
{
    private const string SqlStateSerializationFailure = "40001";
    private const string OC000 = "OC000";
    private const string OC001 = "OC001";

    /// <summary>Default initial wait between retries.</summary>
    public static readonly TimeSpan DefaultInitialWait = TimeSpan.FromMilliseconds(100);

    /// <summary>Default maximum wait between retries.</summary>
    public static readonly TimeSpan DefaultMaxWait = TimeSpan.FromSeconds(5);

    /// <summary>Default backoff multiplier.</summary>
    public const double DefaultMultiplier = 2.0;

    /// <summary>Default maximum retry count for ExecWithRetryAsync.</summary>
    public const int DefaultMaxRetries = 3;

    /// <summary>
    /// Returns true if the exception is an OCC conflict error (SQLSTATE 40001, OC000, or OC001).
    /// </summary>
    public static bool IsOccError(Exception ex)
    {
        if (ex is PostgresException pgEx)
            return IsOccError(pgEx.SqlState, pgEx.MessageText);

        return IsOccError(sqlState: null, message: ex.Message);
    }

    /// <summary>
    /// Testable overload for OCC error detection.
    /// </summary>
    internal static bool IsOccError(string? sqlState, string message)
    {
        if (string.Equals(sqlState, SqlStateSerializationFailure, StringComparison.Ordinal))
            return true;

        return message.Contains(OC000, StringComparison.Ordinal)
            || message.Contains(OC001, StringComparison.Ordinal);
    }

    /// <summary>
    /// Calculates the backoff wait time with jitter.
    /// Returns (waitWithJitter, nextBaseWait).
    /// </summary>
    internal static (TimeSpan wait, TimeSpan nextWait) CalculateBackoff(int attempt, TimeSpan currentWait)
    {
        // Jitter: random [0, wait/4)
        var jitterMs = Random.Shared.Next(0, (int)(currentWait.TotalMilliseconds / 4));
        var wait = currentWait + TimeSpan.FromMilliseconds(jitterMs);

        var nextWait = TimeSpan.FromMilliseconds(currentWait.TotalMilliseconds * DefaultMultiplier);
        if (nextWait > DefaultMaxWait)
            nextWait = DefaultMaxWait;

        return (wait, nextWait);
    }

    /// <summary>
    /// Retries the action on OCC conflict with exponential backoff.
    /// Used by DsqlDataSource.ExecuteAsync.
    /// </summary>
    internal static async Task RetryAsync(
        DsqlDataSource dataSource,
        int maxRetries,
        Func<NpgsqlConnection, Task> action,
        ILogger? logger,
        CancellationToken ct)
    {
        Exception? lastError = null;
        var currentWait = DefaultInitialWait;

        for (int attempt = 0; attempt <= maxRetries; attempt++)
        {
            try
            {
                await using var conn = await dataSource.OpenConnectionAsync(ct).ConfigureAwait(false);
                await action(conn).ConfigureAwait(false);
                return; // success
            }
            catch (Exception ex) when (IsOccError(ex))
            {
                lastError = ex;
                if (attempt < maxRetries)
                {
                    var (wait, nextWait) = CalculateBackoff(attempt, currentWait);
                    logger?.LogWarning(ex,
                        "OCC conflict detected, retrying (attempt {Attempt}/{MaxRetries}, wait {Wait:F2}s)",
                        attempt + 1, maxRetries, wait.TotalSeconds);
                    await Task.Delay(wait, ct).ConfigureAwait(false);
                    currentWait = nextWait;
                }
            }
        }

        throw new DsqlException(
            $"OCC max retries ({maxRetries}) exceeded", lastError!);
    }

    /// <summary>
    /// Retries the action (with return value) on OCC conflict.
    /// </summary>
    internal static async Task<T> RetryAsync<T>(
        DsqlDataSource dataSource,
        int maxRetries,
        Func<NpgsqlConnection, Task<T>> action,
        ILogger? logger,
        CancellationToken ct)
    {
        Exception? lastError = null;
        var currentWait = DefaultInitialWait;

        for (int attempt = 0; attempt <= maxRetries; attempt++)
        {
            try
            {
                await using var conn = await dataSource.OpenConnectionAsync(ct).ConfigureAwait(false);
                return await action(conn).ConfigureAwait(false);
            }
            catch (Exception ex) when (IsOccError(ex))
            {
                lastError = ex;
                if (attempt < maxRetries)
                {
                    var (wait, nextWait) = CalculateBackoff(attempt, currentWait);
                    logger?.LogWarning(ex,
                        "OCC conflict detected, retrying (attempt {Attempt}/{MaxRetries}, wait {Wait:F2}s)",
                        attempt + 1, maxRetries, wait.TotalSeconds);
                    await Task.Delay(wait, ct).ConfigureAwait(false);
                    currentWait = nextWait;
                }
            }
        }

        throw new DsqlException(
            $"OCC max retries ({maxRetries}) exceeded", lastError!);
    }

    /// <summary>
    /// Retries a transaction block with explicit retry configuration.
    /// Manages BEGIN/COMMIT/ROLLBACK via raw SQL — DSQL does not support
    /// Npgsql's BeginTransactionAsync which always sends an isolation level clause.
    /// Opens a fresh connection for each attempt.
    /// </summary>
    public static async Task WithRetryAsync(
        DsqlDataSource dataSource,
        int maxRetries,
        Func<NpgsqlConnection, Task> action,
        ILogger? logger = null,
        CancellationToken ct = default)
    {
        if (maxRetries < 0)
            throw new ArgumentException("maxRetries must be non-negative.", nameof(maxRetries));

        Exception? lastError = null;
        var currentWait = DefaultInitialWait;

        for (int attempt = 0; attempt <= maxRetries; attempt++)
        {
            await using var conn = await dataSource.OpenConnectionAsync(ct).ConfigureAwait(false);
            await using var begin = new NpgsqlCommand("BEGIN", conn);
            await begin.ExecuteNonQueryAsync(ct).ConfigureAwait(false);
            try
            {
                await action(conn).ConfigureAwait(false);
                await using var commit = new NpgsqlCommand("COMMIT", conn);
                await commit.ExecuteNonQueryAsync(ct).ConfigureAwait(false);
                return; // success
            }
            catch (Exception ex) when (IsOccError(ex))
            {
                lastError = ex;
                try
                {
                    await using var rollback = new NpgsqlCommand("ROLLBACK", conn);
                    await rollback.ExecuteNonQueryAsync(ct).ConfigureAwait(false);
                }
                catch { /* already failed */ }

                if (attempt < maxRetries)
                {
                    var (wait, nextWait) = CalculateBackoff(attempt, currentWait);
                    logger?.LogWarning(ex,
                        "OCC conflict detected in transaction, retrying (attempt {Attempt}/{MaxRetries}, wait {Wait:F2}s)",
                        attempt + 1, maxRetries, wait.TotalSeconds);
                    await Task.Delay(wait, ct).ConfigureAwait(false);
                    currentWait = nextWait;
                }
            }
        }

        throw new DsqlException(
            $"OCC max retries ({maxRetries}) exceeded", lastError!);
    }

    /// <summary>
    /// Convenience method: executes a single SQL statement with OCC retry.
    /// Useful for DDL statements like CREATE INDEX ASYNC.
    /// </summary>
    public static async Task ExecWithRetryAsync(
        DsqlDataSource dataSource,
        string sql,
        int maxRetries = DefaultMaxRetries,
        ILogger? logger = null,
        CancellationToken ct = default)
    {
        await RetryAsync(dataSource, maxRetries, async conn =>
        {
            await using var cmd = conn.CreateCommand();
            cmd.CommandText = sql;
            await cmd.ExecuteNonQueryAsync(ct).ConfigureAwait(false);
        }, logger, ct).ConfigureAwait(false);
    }
}
