// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

using System.Text.RegularExpressions;

namespace Amazon.AuroraDsql.Npgsql;

internal static class Util
{
    // Matches *.dsql[*].<region>.on.aws — the [^.]* handles optional suffixes like "dsqlx"
    private static readonly Regex RegionPattern = new(
        @"\.dsql[^.]*\.([^.]+)\.on\.aws$",
        RegexOptions.Compiled);

    // 26 lowercase alphanumeric characters, no dots
    private static readonly Regex ClusterIdPattern = new(
        @"^[a-z0-9]{26}$",
        RegexOptions.Compiled);

    /// <summary>
    /// Extracts the AWS region from a DSQL hostname.
    /// Returns null if the hostname does not match the expected pattern.
    /// </summary>
    internal static string? ParseRegion(string host)
    {
        var match = RegionPattern.Match(host);
        return match.Success ? match.Groups[1].Value : null;
    }

    /// <summary>
    /// Returns true if the input looks like a bare cluster ID (26 lowercase alphanumeric chars).
    /// </summary>
    internal static bool IsClusterId(string? host)
    {
        if (string.IsNullOrEmpty(host))
            return false;
        return ClusterIdPattern.IsMatch(host);
    }

    /// <summary>
    /// Builds a full DSQL hostname from a cluster ID and region.
    /// </summary>
    internal static string BuildHostname(string clusterId, string region)
    {
        return $"{clusterId}.dsql.{region}.on.aws";
    }
}
