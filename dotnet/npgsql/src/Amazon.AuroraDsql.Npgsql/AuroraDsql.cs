// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

namespace Amazon.AuroraDsql.Npgsql;

/// <summary>
/// Entry point for the Aurora DSQL .NET Npgsql connector.
/// </summary>
public static class AuroraDsql
{
    /// <summary>
    /// Creates a connection pool (DsqlDataSource) with IAM token injection and DSQL defaults.
    /// </summary>
    public static DsqlDataSource CreateDataSource(DsqlConfig config)
        => DsqlDataSource.Create(config);

    /// <summary>
    /// Creates a connection pool from a connection string.
    /// </summary>
    public static DsqlDataSource CreateDataSource(string connectionString)
        => DsqlDataSource.Create(connectionString);

    /// <summary>
    /// Creates and opens a single (unpooled) DSQL connection.
    /// </summary>
    public static Task<DsqlConnection> ConnectAsync(DsqlConfig config, CancellationToken ct = default)
        => DsqlConnection.ConnectAsync(config, ct);

    /// <summary>
    /// Creates and opens a single connection from a connection string.
    /// </summary>
    public static Task<DsqlConnection> ConnectAsync(string connectionString, CancellationToken ct = default)
        => DsqlConnection.ConnectAsync(connectionString, ct);
}
