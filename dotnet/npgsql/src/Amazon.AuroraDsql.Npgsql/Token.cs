// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

using Amazon;
using Amazon.DSQL.Util;
using Amazon.Runtime;
using Amazon.Runtime.CredentialManagement;

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
        RegionEndpoint region)
    {
        return IsAdminUser(user)
            ? DSQLAuthTokenGenerator.GenerateDbConnectAdminAuthToken(credentials, region, host)
            : DSQLAuthTokenGenerator.GenerateDbConnectAuthToken(credentials, region, host);
    }

    /// <summary>
    /// Resolves AWS credentials from the config's credential chain.
    /// Order: CustomCredentialsProvider > Profile > SDK default chain.
    /// </summary>
    internal static AWSCredentials ResolveCredentials(ResolvedConfig config)
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

        // SDK default credential chain (synchronous).
        // FallbackCredentialsFactory is deprecated in AWSSDK.Core v4 but the replacement
        // (DefaultAWSCredentialsIdentityResolver) requires async and returns BaseIdentity.
        // Suppress until the DSQL token generator SDK accepts the new identity type.
#pragma warning disable CS0618
        return FallbackCredentialsFactory.GetCredentials();
#pragma warning restore CS0618
    }
}
