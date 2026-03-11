// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

using Amazon;
using Amazon.Runtime;
using Npgsql;

namespace Amazon.AuroraDsql.Npgsql;

/// <summary>
/// A single Aurora DSQL connection (no pooling). For scripts and simple use cases.
/// Use <see cref="DsqlDataSource"/> for connection pooling.
/// </summary>
public sealed class DsqlConnection : IAsyncDisposable, IDisposable
{
    private readonly NpgsqlConnection _inner;
    private readonly NpgsqlDataSource _dataSource;

    private DsqlConnection(NpgsqlConnection inner, NpgsqlDataSource dataSource)
    {
        _inner = inner;
        _dataSource = dataSource;
    }

    /// <summary>
    /// Creates and opens a single DSQL connection with a fresh IAM token.
    /// Uses NpgsqlDataSourceBuilder with Pooling=false to get access to
    /// UsePasswordProvider and UseSslClientAuthenticationOptionsCallback.
    /// </summary>
    public static async Task<DsqlConnection> ConnectAsync(DsqlConfig config, CancellationToken ct = default)
    {
        var resolved = config.Resolve();
        var credentials = Token.ResolveCredentials(resolved);
        var regionEndpoint = RegionEndpoint.GetBySystemName(resolved.Region);

        var csb = BuildConnectionStringBuilder(resolved);
        var builder = new NpgsqlDataSourceBuilder(csb.ConnectionString);

        DsqlDataSource.ConfigureBuilder(builder, resolved, credentials, regionEndpoint);

        var dataSource = builder.Build();
        NpgsqlConnection? conn = null;
        try
        {
            conn = await dataSource.OpenConnectionAsync(ct).ConfigureAwait(false);
        }
        catch
        {
            conn?.Dispose();
            dataSource.Dispose();
            throw;
        }
        return new DsqlConnection(conn, dataSource);
    }

    /// <summary>
    /// Creates and opens a single DSQL connection from a connection string.
    /// </summary>
    public static Task<DsqlConnection> ConnectAsync(string connectionString, CancellationToken ct = default)
    {
        var config = DsqlConfig.FromConnectionString(connectionString);
        return ConnectAsync(config, ct);
    }

    /// <summary>
    /// Builds the connection string for a single (unpooled) connection.
    /// Exposed as internal for unit testing.
    /// </summary>
    internal static NpgsqlConnectionStringBuilder BuildConnectionStringBuilder(ResolvedConfig config)
    {
        return new NpgsqlConnectionStringBuilder
        {
            Host = config.Host,
            Port = config.Port,
            Database = config.Database,
            Username = config.User,
            SslMode = SslMode.VerifyFull,
            SslNegotiation = SslNegotiation.Direct,
            ApplicationName = config.ApplicationName,
            Pooling = false,
            Enlist = false,
        };
    }

    // --- Delegation of common NpgsqlConnection methods ---

    /// <summary>Creates a command on this connection.</summary>
    public NpgsqlCommand CreateCommand(string? commandText = null)
    {
        var cmd = _inner.CreateCommand();
        if (commandText != null) cmd.CommandText = commandText;
        return cmd;
    }

    /// <summary>Exposes the underlying NpgsqlConnection for advanced use.</summary>
    public NpgsqlConnection InnerConnection => _inner;

    /// <inheritdoc />
    public void Dispose()
    {
        _inner.Dispose();
        _dataSource.Dispose();
    }

    /// <inheritdoc />
    public async ValueTask DisposeAsync()
    {
        await _inner.DisposeAsync().ConfigureAwait(false);
        await _dataSource.DisposeAsync().ConfigureAwait(false);
    }
}
