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
    public static async Task<DsqlDataSource> CreateAsync(DsqlConfig config)
    {
        ArgumentNullException.ThrowIfNull(config);
        var resolved = config.ResolveInternal();
        var credentials = await Token.ResolveCredentialsAsync(resolved).ConfigureAwait(false);
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
    public static Task<DsqlDataSource> CreateAsync(string connectionString)
    {
        var config = DsqlConfig.FromConnectionString(connectionString);
        return CreateAsync(config);
    }

    /// <summary>
    /// Builds the NpgsqlConnectionStringBuilder with all DSQL defaults applied.
    /// </summary>
    internal static NpgsqlConnectionStringBuilder BuildConnectionStringBuilder(ResolvedConfig config)
    {
        var csb = DsqlConnection.BuildBaseConnectionStringBuilder(config);
        csb.MaxPoolSize = config.MaxPoolSize;
        csb.MinPoolSize = config.MinPoolSize;
        csb.ConnectionLifetime = config.ConnectionLifetime;
        csb.ConnectionIdleLifetime = config.ConnectionIdleLifetime;
        csb.NoResetOnClose = true; // DSQL manages session state automatically
        return csb;
    }

    /// <summary>
    /// Configures the data source builder with IAM password provider and TLS settings.
    /// Shared by DsqlDataSource.CreateAsync and DsqlConnection.ConnectAsync.
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

        // Defense-in-depth: .NET 8+ defaults to TLS 1.2+, but we pin explicitly
        // in case the connector is used in an environment with a weaker default.
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
    /// Executes an action with a connection from the pool.
    /// </summary>
    /// <param name="action">The async action to execute with a pooled connection.</param>
    /// <param name="ct">Cancellation token.</param>
    public async Task ExecuteAsync(
        Func<NpgsqlConnection, Task> action,
        CancellationToken ct = default)
    {
        await using var conn = await OpenConnectionAsync(ct).ConfigureAwait(false);
        await action(conn).ConfigureAwait(false);
    }

    /// <summary>
    /// Executes an action with a return value using a connection from the pool.
    /// </summary>
    /// <param name="action">The async action to execute with a pooled connection.</param>
    /// <param name="ct">Cancellation token.</param>
    public async Task<T> ExecuteAsync<T>(
        Func<NpgsqlConnection, Task<T>> action,
        CancellationToken ct = default)
    {
        await using var conn = await OpenConnectionAsync(ct).ConfigureAwait(false);
        return await action(conn).ConfigureAwait(false);
    }

    /// <summary>
    /// Executes an action inside a transaction with OCC retry.
    /// Manages BEGIN/COMMIT/ROLLBACK automatically. On OCC conflict,
    /// rolls back and re-executes with a fresh connection.
    /// </summary>
    /// <param name="action">The async action to execute within the transaction.</param>
    /// <param name="maxOccRetries">
    /// Maximum OCC retry attempts. Overrides <see cref="DsqlConfig.OccMaxRetries"/>.
    /// Pass null to use the config default, or 0 to disable retry.
    /// </param>
    /// <param name="ct">Cancellation token.</param>
    public async Task WithTransactionRetryAsync(
        Func<NpgsqlConnection, Task> action,
        int? maxOccRetries = null,
        CancellationToken ct = default)
    {
        var maxRetries = ResolveRetryCount(maxOccRetries);
        await OccRetry.WithTransactionRetryAsync(
            _inner, maxRetries, action, _logger, ct).ConfigureAwait(false);
    }

    /// <summary>
    /// Executes a single SQL statement with OCC retry.
    /// Useful for DDL statements like CREATE TABLE or CREATE INDEX ASYNC.
    /// </summary>
    /// <param name="sql">The SQL statement to execute.</param>
    /// <param name="maxOccRetries">
    /// Maximum OCC retry attempts. Overrides <see cref="DsqlConfig.OccMaxRetries"/>.
    /// Pass null to use the config default, or 0 to disable retry.
    /// </param>
    /// <param name="ct">Cancellation token.</param>
    public async Task ExecWithRetryAsync(
        string sql,
        int? maxOccRetries = null,
        CancellationToken ct = default)
    {
        var maxRetries = ResolveRetryCount(maxOccRetries);
        await OccRetry.ExecWithRetryAsync(
            _inner, sql, maxRetries, _logger, ct).ConfigureAwait(false);
    }

    /// <summary>Exposes the underlying NpgsqlDataSource for advanced use.</summary>
    public NpgsqlDataSource DataSource => _inner;

    /// <inheritdoc />
    public void Dispose() => _inner.Dispose();

    /// <inheritdoc />
    public ValueTask DisposeAsync() => _inner.DisposeAsync();

    private int ResolveRetryCount(int? maxOccRetries)
    {
        if (maxOccRetries < 0)
            throw new ArgumentException("maxOccRetries must be null, 0, or a positive integer.", nameof(maxOccRetries));
        return maxOccRetries ?? _config.OccMaxRetries ?? 0;
    }
}
