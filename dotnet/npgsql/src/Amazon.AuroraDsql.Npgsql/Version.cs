// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

using System.Reflection;

namespace Amazon.AuroraDsql.Npgsql;

internal static class ConnectorVersion
{
    internal const string Default = "0.0.0";

    internal static string Current { get; } = GetVersion();

    internal static string ApplicationName { get; } = $"aurora-dsql-dotnet-npgsql/{Current}";

    internal static string BuildApplicationName(string? ormPrefix)
    {
        if (!string.IsNullOrWhiteSpace(ormPrefix))
            return $"{ormPrefix.Trim()}:{ApplicationName}";
        return ApplicationName;
    }

    private static string GetVersion()
    {
        var attr = typeof(ConnectorVersion).Assembly
            .GetCustomAttribute<AssemblyInformationalVersionAttribute>();
        if (attr?.InformationalVersion is { Length: > 0 } v)
        {
            // Strip metadata suffix like "+sha" if present
            var plus = v.IndexOf('+');
            return plus > 0 ? v[..plus] : v;
        }
        return Default;
    }
}
