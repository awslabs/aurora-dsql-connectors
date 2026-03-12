// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

namespace Amazon.AuroraDsql.Npgsql;

/// <summary>
/// Exception thrown by the Aurora DSQL connector.
/// </summary>
public sealed class DsqlException : Exception
{
    /// <inheritdoc />
    public DsqlException(string message) : base(message) { }

    /// <inheritdoc />
    public DsqlException(string message, Exception innerException) : base(message, innerException) { }
}
