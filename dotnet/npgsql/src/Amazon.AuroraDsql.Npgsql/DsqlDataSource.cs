// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

using System.Security.Authentication;
using Amazon;
using Amazon.Runtime;
using Microsoft.Extensions.Logging;
using Npgsql;

namespace Amazon.AuroraDsql.Npgsql;

/// <summary>
/// Aurora DSQL connection pool backed by NpgsqlDataSource.
/// Injects fresh IAM tokens per physical connection, enforces SSL, and provides opt-in OCC retry.
/// </summary>
public sealed class DsqlDataSource : IAsyncDisposable, IDisposable
{
    private readonly NpgsqlDataSource _inner;
    private readonly ResolvedConfig _config;
    private readonly ILogger? _logger;

    private DsqlDataSource(NpgsqlDataSource inner, ResolvedConfig config)
    {
        _inner = inner;
        _config = config;
        _logger = config.LoggerFactory?.CreateLogger<DsqlDataSource>();
    }

    /// <summary>
    /// Creates a new DsqlDataSource from the given config.
    /// Resolves AWS credentials once; generates fresh IAM tokens per physical connection.
    /// </summary>
    public static DsqlDataSource Create(DsqlConfig config)
    {
        var resolved = config.Resolve();
        var credentials = Token.ResolveCredentials(resolved);
        var regionEndpoint = RegionEndpoint.GetBySystemName(resolved.Region);

        var csb = BuildConnectionStringBuilder(resolved);
        var builder = new NpgsqlDataSourceBuilder(csb.ConnectionString);

        ConfigureBuilder(builder, resolved, credentials, regionEndpoint);

        if (resolved.LoggerFactory != null)
            builder.UseLoggerFactory(resolved.LoggerFactory);

        return new DsqlDataSource(builder.Build(), resolved);
    }

    /// <summary>
    /// Creates a new DsqlDataSource from a connection string.
    /// </summary>
    public static DsqlDataSource Create(string connectionString)
    {
        var config = DsqlConfig.FromConnectionString(connectionString);
        return Create(config);
    }

    /// <summary>
    /// Builds the NpgsqlConnectionStringBuilder with all DSQL defaults applied.
    /// Exposed as internal for unit testing.
    /// </summary>
    internal static NpgsqlConnectionStringBuilder BuildConnectionStringBuilder(ResolvedConfig config)
    {
        var csb = DsqlConnection.BuildBaseConnectionStringBuilder(config);
        csb.MaxPoolSize = config.MaxPoolSize;
        csb.MinPoolSize = config.MinPoolSize;
        csb.ConnectionLifetime = config.ConnectionLifetime;
        csb.ConnectionIdleLifetime = config.ConnectionIdleLifetime;
        csb.NoResetOnClose = true; // DSQL does not support DISCARD ALL
        return csb;
    }

    /// <summary>
    /// Configures the data source builder with IAM password provider and TLS settings.
    /// Shared by DsqlDataSource.Create and DsqlConnection.ConnectAsync.
    /// </summary>
    internal static void ConfigureBuilder(
        NpgsqlDataSourceBuilder builder,
        ResolvedConfig resolved,
        AWSCredentials credentials,
        RegionEndpoint regionEndpoint)
    {
        builder.UsePasswordProvider(
            passwordProvider: (_) =>
                Token.GenerateToken(resolved.Host, resolved.User, credentials, regionEndpoint),
            passwordProviderAsync: (_, _) =>
                new ValueTask<string>(
                    Token.GenerateToken(resolved.Host, resolved.User, credentials, regionEndpoint)));

        builder.UseSslClientAuthenticationOptionsCallback(options =>
        {
            options.EnabledSslProtocols = SslProtocols.Tls12 | SslProtocols.Tls13;
        });
    }

    // --- Delegation of NpgsqlDataSource API ---

    /// <summary>Creates an unopened connection from the pool.</summary>
    public NpgsqlConnection CreateConnection() => _inner.CreateConnection();

    /// <summary>Opens a connection from the pool.</summary>
    public ValueTask<NpgsqlConnection> OpenConnectionAsync(CancellationToken ct = default)
        => _inner.OpenConnectionAsync(ct);

    /// <summary>Creates a command bound to a connection from the pool.</summary>
    public NpgsqlCommand CreateCommand(string? commandText = null)
        => _inner.CreateCommand(commandText);

    /// <summary>Creates a batch bound to a connection from the pool.</summary>
    public NpgsqlBatch CreateBatch()
        => _inner.CreateBatch();

    /// <summary>
    /// Executes an action with a connection from the pool, with opt-in OCC retry.
    /// When retry is enabled, the action is re-executed from scratch on OCC conflict
    /// (fresh connection each attempt). The action MUST be safe to retry — either
    /// wrap writes in a transaction (BEGIN/COMMIT) or ensure idempotency.
    /// </summary>
    public async Task ExecuteAsync(
        Func<NpgsqlConnection, Task> action,
        int? retryOcc = null,
        CancellationToken ct = default)
    {
        var maxRetries = ResolveRetryCount(retryOcc);

        if (maxRetries <= 0)
        {
            await using var conn = await OpenConnectionAsync(ct).ConfigureAwait(false);
            await action(conn).ConfigureAwait(false);
            return;
        }

        await OccRetry.RetryAsync(
            this, maxRetries, action, _logger, ct).ConfigureAwait(false);
    }

    /// <summary>
    /// Executes an action with a return value, with opt-in OCC retry.
    /// When retry is enabled, the action is re-executed from scratch on OCC conflict
    /// (fresh connection each attempt). The action MUST be safe to retry — either
    /// wrap writes in a transaction (BEGIN/COMMIT) or ensure idempotency.
    /// </summary>
    public async Task<T> ExecuteAsync<T>(
        Func<NpgsqlConnection, Task<T>> action,
        int? retryOcc = null,
        CancellationToken ct = default)
    {
        var maxRetries = ResolveRetryCount(retryOcc);

        if (maxRetries <= 0)
        {
            await using var conn = await OpenConnectionAsync(ct).ConfigureAwait(false);
            return await action(conn).ConfigureAwait(false);
        }

        return await OccRetry.RetryAsync(
            this, maxRetries, action, _logger, ct).ConfigureAwait(false);
    }

    /// <summary>Exposes the underlying NpgsqlDataSource for advanced use.</summary>
    public NpgsqlDataSource InnerDataSource => _inner;

    /// <inheritdoc />
    public void Dispose() => _inner.Dispose();

    /// <inheritdoc />
    public ValueTask DisposeAsync() => _inner.DisposeAsync();

    private int ResolveRetryCount(int? retryOcc)
    {
        if (retryOcc < 0)
            throw new ArgumentException("retryOcc must be null, 0, or a positive integer.", nameof(retryOcc));
        return retryOcc ?? _config.OccMaxRetries ?? 0;
    }
}
