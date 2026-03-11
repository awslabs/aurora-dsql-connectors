// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

using Npgsql;
using Amazon.AuroraDsql.Npgsql;
using Xunit;

namespace Amazon.AuroraDsql.Npgsql.Tests;

public class OccRetryTests
{
    // --- IsOccError ---

    [Fact]
    public void IsOccError_SqlState40001_ReturnsTrue()
    {
        Assert.True(OccRetry.IsOccError(sqlState: "40001", message: ""));
    }

    [Fact]
    public void IsOccError_OC000InMessage_ReturnsTrue()
    {
        Assert.True(OccRetry.IsOccError(sqlState: null, message: "ERROR: OC000 mutation conflict"));
    }

    [Fact]
    public void IsOccError_OC001InMessage_ReturnsTrue()
    {
        Assert.True(OccRetry.IsOccError(sqlState: null, message: "ERROR: OC001 schema conflict"));
    }

    [Fact]
    public void IsOccError_UnrelatedError_ReturnsFalse()
    {
        Assert.False(OccRetry.IsOccError(sqlState: "23505", message: "unique violation"));
    }

    [Fact]
    public void IsOccError_NullSqlStateNoMatch_ReturnsFalse()
    {
        Assert.False(OccRetry.IsOccError(sqlState: null, message: "some other error"));
    }

    // --- CalculateBackoff ---

    [Fact]
    public void CalculateBackoff_FirstAttempt_ReturnsInitialWait()
    {
        var (wait, _) = OccRetry.CalculateBackoff(attempt: 0, currentWait: TimeSpan.FromMilliseconds(100));
        // wait should be >= 100ms (base) and <= 124ms (base + max jitter: Next(0, 25) returns [0,24])
        Assert.InRange(wait.TotalMilliseconds, 100, 124);
    }

    [Fact]
    public void CalculateBackoff_NextWaitDoubles()
    {
        var (_, nextWait) = OccRetry.CalculateBackoff(attempt: 0, currentWait: TimeSpan.FromMilliseconds(100));
        Assert.Equal(200, nextWait.TotalMilliseconds);
    }

    [Fact]
    public void CalculateBackoff_CapsAtMaxWait()
    {
        var (_, nextWait) = OccRetry.CalculateBackoff(attempt: 0, currentWait: TimeSpan.FromSeconds(4));
        Assert.Equal(5000, nextWait.TotalMilliseconds); // capped at 5s
    }
}
