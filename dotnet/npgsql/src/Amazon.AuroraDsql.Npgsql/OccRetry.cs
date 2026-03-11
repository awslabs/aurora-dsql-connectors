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

    internal static readonly TimeSpan DefaultInitialWait = TimeSpan.FromMilliseconds(100);
    internal static readonly TimeSpan DefaultMaxWait = TimeSpan.FromSeconds(5);
    internal const double DefaultMultiplier = 2.0;

    /// <summary>Default maximum retry attempts for OCC conflicts.</summary>
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
    internal static (TimeSpan wait, TimeSpan nextWait) CalculateBackoff(TimeSpan currentWait)
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
    /// Core retry loop with exponential backoff. All retry methods delegate here.
    /// The <paramref name="attempt"/> delegate runs one attempt and returns a result.
    /// It should throw on OCC errors; non-OCC exceptions propagate immediately.
    /// </summary>
    private static async Task<T> RetryCoreAsync<T>(
        int maxRetries,
        Func<Task<T>> attempt,
        string logPrefix,
        ILogger? logger,
        CancellationToken ct)
    {
        Exception? lastError = null;
        var currentWait = DefaultInitialWait;

        for (int i = 0; i <= maxRetries; i++)
        {
            try
            {
                return await attempt().ConfigureAwait(false);
            }
            catch (Exception ex) when (IsOccError(ex))
            {
                lastError = ex;
                if (i < maxRetries)
                {
                    var (wait, nextWait) = CalculateBackoff(currentWait);
                    logger?.LogWarning(ex,
                        "OCC conflict detected{Prefix}, retrying (attempt {Attempt}/{MaxRetries}, wait {Wait:F2}s)",
                        logPrefix, i + 1, maxRetries, wait.TotalSeconds);
                    await Task.Delay(wait, ct).ConfigureAwait(false);
                    currentWait = nextWait;
                }
            }
        }

        throw new DsqlException(
            $"OCC max retries ({maxRetries}) exceeded", lastError!);
    }

    /// <summary>
    /// Retries the action on OCC conflict with exponential backoff.
    /// Used by DsqlDataSource.ExecuteAsync.
    /// </summary>
    internal static Task RetryAsync(
        DsqlDataSource dataSource,
        int maxRetries,
        Func<NpgsqlConnection, Task> action,
        ILogger? logger,
        CancellationToken ct)
    {
        return RetryAsync(dataSource.DataSource, maxRetries, action, logger, ct);
    }

    /// <summary>
    /// Retries the action (with return value) on OCC conflict.
    /// </summary>
    internal static Task<T> RetryAsync<T>(
        DsqlDataSource dataSource,
        int maxRetries,
        Func<NpgsqlConnection, Task<T>> action,
        ILogger? logger,
        CancellationToken ct)
    {
        return RetryAsync(dataSource.DataSource, maxRetries, action, logger, ct);
    }

    private static Task RetryAsync(
        NpgsqlDataSource npgsqlDataSource,
        int maxRetries,
        Func<NpgsqlConnection, Task> action,
        ILogger? logger,
        CancellationToken ct)
    {
        return RetryCoreAsync<object?>(maxRetries, async () =>
        {
            await using var conn = await npgsqlDataSource.OpenConnectionAsync(ct).ConfigureAwait(false);
            await action(conn).ConfigureAwait(false);
            return null;
        }, "", logger, ct);
    }

    private static Task<T> RetryAsync<T>(
        NpgsqlDataSource npgsqlDataSource,
        int maxRetries,
        Func<NpgsqlConnection, Task<T>> action,
        ILogger? logger,
        CancellationToken ct)
    {
        return RetryCoreAsync(maxRetries, async () =>
        {
            await using var conn = await npgsqlDataSource.OpenConnectionAsync(ct).ConfigureAwait(false);
            return await action(conn).ConfigureAwait(false);
        }, "", logger, ct);
    }

    /// <summary>
    /// Retries a transaction block with explicit retry configuration.
    /// Manages BEGIN/COMMIT/ROLLBACK via raw SQL because DSQL uses fixed
    /// Repeatable Read isolation — Npgsql's BeginTransactionAsync sends an
    /// explicit isolation level clause (e.g., "BEGIN TRANSACTION ISOLATION
    /// LEVEL READ COMMITTED") that is unnecessary here.
    /// Opens a fresh connection for each attempt.
    /// </summary>
    public static Task WithTransactionRetryAsync(
        DsqlDataSource dataSource,
        int maxRetries,
        Func<NpgsqlConnection, Task> action,
        ILogger? logger = null,
        CancellationToken ct = default)
    {
        return WithTransactionRetryAsync(dataSource.DataSource, maxRetries, action, logger, ct);
    }

    /// <summary>
    /// Retries a transaction block using a raw NpgsqlDataSource.
    /// Use this overload when you have an NpgsqlDataSource from manual setup
    /// or dependency injection rather than a DsqlDataSource.
    /// </summary>
    public static async Task WithTransactionRetryAsync(
        NpgsqlDataSource npgsqlDataSource,
        int maxRetries,
        Func<NpgsqlConnection, Task> action,
        ILogger? logger = null,
        CancellationToken ct = default)
    {
        if (maxRetries < 0)
            throw new ArgumentException("maxRetries must be non-negative.", nameof(maxRetries));

        await RetryCoreAsync<object?>(maxRetries, async () =>
        {
            await using var conn = await npgsqlDataSource.OpenConnectionAsync(ct).ConfigureAwait(false);
            await using var begin = new NpgsqlCommand("BEGIN", conn);
            await begin.ExecuteNonQueryAsync(ct).ConfigureAwait(false);
            try
            {
                await action(conn).ConfigureAwait(false);
                await using var commit = new NpgsqlCommand("COMMIT", conn);
                await commit.ExecuteNonQueryAsync(ct).ConfigureAwait(false);
                return null;
            }
            catch
            {
                // Best-effort rollback; the connection is discarded after this attempt anyway.
                // Use CancellationToken.None so the rollback succeeds even if the caller cancelled.
                try
                {
                    await using var rollback = new NpgsqlCommand("ROLLBACK", conn);
                    await rollback.ExecuteNonQueryAsync(CancellationToken.None).ConfigureAwait(false);
                }
                catch { /* connection may already be broken */ }
                throw;
            }
        }, " in transaction", logger, ct).ConfigureAwait(false);
    }

    /// <summary>
    /// Convenience method: executes a single SQL statement with OCC retry.
    /// Useful for DDL statements like CREATE INDEX ASYNC.
    /// </summary>
    public static Task ExecWithRetryAsync(
        DsqlDataSource dataSource,
        string sql,
        int maxRetries = DefaultMaxRetries,
        ILogger? logger = null,
        CancellationToken ct = default)
    {
        return ExecWithRetryAsync(dataSource.DataSource, sql, maxRetries, logger, ct);
    }

    /// <summary>
    /// Executes a single SQL statement with OCC retry using a raw NpgsqlDataSource.
    /// Use this overload when you have an NpgsqlDataSource from manual setup
    /// or dependency injection rather than a DsqlDataSource.
    /// </summary>
    public static async Task ExecWithRetryAsync(
        NpgsqlDataSource npgsqlDataSource,
        string sql,
        int maxRetries = DefaultMaxRetries,
        ILogger? logger = null,
        CancellationToken ct = default)
    {
        if (maxRetries < 0)
            throw new ArgumentException("maxRetries must be non-negative.", nameof(maxRetries));

        await RetryAsync(npgsqlDataSource, maxRetries, async conn =>
        {
            await using var cmd = conn.CreateCommand();
            cmd.CommandText = sql;
            await cmd.ExecuteNonQueryAsync(ct).ConfigureAwait(false);
        }, logger, ct).ConfigureAwait(false);
    }
}
