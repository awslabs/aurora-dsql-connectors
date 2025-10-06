/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License").
 * You may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package software.amazon.dsql.jdbc;

import java.sql.SQLException;
import java.time.Duration;
import java.time.Instant;
import java.util.Objects;
import java.util.concurrent.ConcurrentHashMap;
import java.util.function.Consumer;
import java.util.logging.Level;
import java.util.logging.Logger;
import javax.annotation.Nonnull;
import software.amazon.awssdk.auth.credentials.AwsCredentialsProvider;
import software.amazon.awssdk.regions.Region;
import software.amazon.awssdk.services.dsql.DsqlUtilities;
import software.amazon.awssdk.services.dsql.model.GenerateAuthTokenRequest;

/** Static utility class for managing Aurora DSQL authentication tokens with shared caching. */
public final class TokenManager {

    private static final Logger LOGGER = Logger.getLogger(TokenManager.class.getName());

    // Shared token cache across all connections
    private static final ConcurrentHashMap<TokenKey, CachedToken> TOKEN_CACHE =
            new ConcurrentHashMap<>();

    // Token refresh buffer - refresh when 20% of lifetime remains
    private static final double REFRESH_BUFFER_PERCENTAGE = 0.2;

    // Private constructor to prevent instantiation
    private TokenManager() {
        throw new UnsupportedOperationException("TokenManager is a utility class");
    }

    /** Gets a valid authentication token, using cache or generating new one as needed. */
    @Nonnull
    public static String getToken(
            @Nonnull final String hostname,
            @Nonnull final Region region,
            @Nonnull final String user,
            @Nonnull final AwsCredentialsProvider credentialsProvider,
            final Duration tokenDuration)
            throws SQLException {
        final TokenKey key =
                new TokenKey(hostname, region, user, credentialsProvider, tokenDuration);

        // Check cache first
        final CachedToken cached = TOKEN_CACHE.get(key);
        if (cached != null && !cached.isExpiredOrExpiringSoon(REFRESH_BUFFER_PERCENTAGE)) {
            LOGGER.log(Level.FINE, "Aurora DSQL: Returning cached token for user: {0}", user);
            return cached.token;
        }

        // Need to refresh
        LOGGER.log(Level.FINE, "Aurora DSQL: Generating new token for user: {0}", user);
        return refreshToken(key);
    }

    /** Forces a token refresh for the given configuration. */
    @Nonnull
    public static String forceRefreshToken(
            @Nonnull final String hostname,
            @Nonnull final Region region,
            @Nonnull final String user,
            @Nonnull final AwsCredentialsProvider credentialsProvider,
            final Duration tokenDuration)
            throws SQLException {
        final TokenKey key =
                new TokenKey(hostname, region, user, credentialsProvider, tokenDuration);
        LOGGER.log(Level.FINE, "Forcing token refresh for user: {0}", user);
        return refreshToken(key);
    }

    /** Clears all cached tokens. */
    public static void clearCache() {
        LOGGER.log(Level.FINE, "Clearing all token cache");
        TOKEN_CACHE.clear();
    }

    @Nonnull
    private static String refreshToken(@Nonnull final TokenKey key) throws SQLException {
        try {
            LOGGER.log(
                    Level.FINE,
                    "Aurora DSQL: Generating new authentication token for user: {0}",
                    key.user);

            final DsqlUtilities utilities =
                    DsqlUtilities.builder()
                            .region(key.region)
                            .credentialsProvider(key.credentialsProvider)
                            .build();

            final Consumer<GenerateAuthTokenRequest.Builder> requester =
                    builder -> {
                        builder.hostname(key.hostname).region(key.region);

                        // Only add tokenDuration if it's not null
                        if (key.tokenDuration != null) {
                            builder.expiresIn(key.tokenDuration);
                        }
                    };

            final String token;
            if ("admin".equals(key.user)) {
                token = utilities.generateDbConnectAdminAuthToken(requester);
            } else {
                token = utilities.generateDbConnectAuthToken(requester);
            }

            final Instant now = Instant.now();
            // if token duration is null, then cache it for 15 minutes following dsql limit
            // https://docs.aws.amazon.com/aurora-dsql/latest/userguide/SECTION_authentication-token.html
            final Duration cachedTokenDuration =
                    key.tokenDuration == null ? Duration.ofMinutes(15) : key.tokenDuration;
            final CachedToken cachedToken = new CachedToken(token, now, cachedTokenDuration);
            TOKEN_CACHE.put(key, cachedToken);

            // Record token refresh for monitoring
            LOGGER.log(
                    Level.FINE,
                    "Aurora DSQL: Token generated successfully. Duration: {0} seconds, Expires at: {1}",
                    new Object[] {cachedTokenDuration.getSeconds(), cachedToken.expiresAt});

            return token;

        } catch (Exception e) {
            LOGGER.log(Level.SEVERE, "Aurora DSQL: Failed to generate authentication token", e);
            throw new SQLException("Token generation failed: " + e.getMessage(), e);
        }
    }

    /** Key for token caching based on configuration. */
    private static class TokenKey {
        private final String hostname;
        private final Region region;
        private final String user;
        private final AwsCredentialsProvider credentialsProvider;
        private final Duration tokenDuration;

        TokenKey(
                String hostname,
                Region region,
                String user,
                AwsCredentialsProvider credentialsProvider,
                Duration tokenDuration) {
            this.hostname = hostname;
            this.region = region;
            this.user = user;
            this.credentialsProvider = credentialsProvider;
            this.tokenDuration = tokenDuration;
        }

        @Override
        public boolean equals(Object o) {
            if (this == o) return true;
            if (o == null || getClass() != o.getClass()) return false;
            TokenKey tokenKey = (TokenKey) o;
            return Objects.equals(hostname, tokenKey.hostname)
                    && Objects.equals(region, tokenKey.region)
                    && Objects.equals(user, tokenKey.user)
                    && Objects.equals(credentialsProvider, tokenKey.credentialsProvider)
                    && Objects.equals(tokenDuration, tokenKey.tokenDuration);
        }

        @Override
        public int hashCode() {
            return Objects.hash(hostname, region, user, credentialsProvider, tokenDuration);
        }
    }

    /** Represents a cached token with its metadata. */
    private static class CachedToken {
        final String token;
        final Instant generatedAt;
        final Instant expiresAt;

        CachedToken(String token, Instant generatedAt, Duration duration) {
            this.token = token;
            this.generatedAt = generatedAt;
            this.expiresAt = generatedAt.plus(duration);
        }

        boolean isExpiredOrExpiringSoon(double bufferPercentage) {
            final Instant now = Instant.now();
            final Duration totalLifetime = Duration.between(generatedAt, expiresAt);
            final Duration bufferTime =
                    totalLifetime.multipliedBy((long) (bufferPercentage * 100)).dividedBy(100);
            final Instant refreshThreshold = expiresAt.minus(bufferTime);

            return now.isAfter(refreshThreshold);
        }
    }
}
