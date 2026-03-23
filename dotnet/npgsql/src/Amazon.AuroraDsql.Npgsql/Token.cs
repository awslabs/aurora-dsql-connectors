// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

using Amazon;
using Amazon.DSQL.Util;
using Amazon.Runtime;
using Amazon.Runtime.CredentialManagement;
using Amazon.Runtime.Credentials;

namespace Amazon.AuroraDsql.Npgsql;

internal static class Token
{
    private const string AdminUser = "admin";

    internal static bool IsAdminUser(string user) =>
        string.Equals(user, AdminUser, StringComparison.Ordinal);

    /// <summary>
    /// Generates a fresh IAM auth token for the given host and user.
    /// This is a local SigV4 presigning operation — no network calls.
    /// </summary>
    internal static string GenerateToken(
        string host,
        string user,
        AWSCredentials credentials,
        RegionEndpoint region,
        int? tokenDurationSecs = null)
    {
        if (tokenDurationSecs.HasValue)
        {
            var expiresIn = TimeSpan.FromSeconds(tokenDurationSecs.Value);
            return IsAdminUser(user)
                ? DSQLAuthTokenGenerator.GenerateDbConnectAdminAuthToken(credentials, region, host, expiresIn)
                : DSQLAuthTokenGenerator.GenerateDbConnectAuthToken(credentials, region, host, expiresIn);
        }

        return IsAdminUser(user)
            ? DSQLAuthTokenGenerator.GenerateDbConnectAdminAuthToken(credentials, region, host)
            : DSQLAuthTokenGenerator.GenerateDbConnectAuthToken(credentials, region, host);
    }

    /// <summary>
    /// Resolves AWS credentials from the config's credential chain.
    /// Order: CustomCredentialsProvider > Profile > SDK default chain.
    /// </summary>
    internal static async Task<AWSCredentials> ResolveCredentialsAsync(ResolvedConfig config)
    {
        if (config.CustomCredentialsProvider != null)
            return config.CustomCredentialsProvider;

        if (!string.IsNullOrWhiteSpace(config.Profile))
        {
            var chain = new CredentialProfileStoreChain();
            if (chain.TryGetAWSCredentials(config.Profile, out var profileCredentials))
                return profileCredentials;
            throw new DsqlException($"AWS profile '{config.Profile}' not found or has no credentials.");
        }

        return await DefaultAWSCredentialsIdentityResolver.GetCredentialsAsync().ConfigureAwait(false);
    }
}
