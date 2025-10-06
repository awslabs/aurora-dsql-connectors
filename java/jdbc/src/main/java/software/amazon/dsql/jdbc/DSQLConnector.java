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

import edu.umd.cs.findbugs.annotations.SuppressFBWarnings;
import java.net.URISyntaxException;
import java.sql.Connection;
import java.sql.Driver;
import java.sql.DriverManager;
import java.sql.DriverPropertyInfo;
import java.sql.SQLException;
import java.time.Duration;
import java.util.Collection;
import java.util.Properties;
import java.util.logging.Logger;
import javax.annotation.Nonnull;
import javax.annotation.Nullable;
import software.amazon.awssdk.auth.credentials.AwsCredentialsProvider;
import software.amazon.awssdk.auth.credentials.ProfileCredentialsProvider;
import software.amazon.awssdk.regions.Region;

/**
 * JDBC {@link Driver} implementation that intercepts connection requests and injects IAM
 * authentication tokens.
 *
 * <p>This class serves as the main entry point for JDBC connections to Aurora DSQL. It implements
 * the standard JDBC {@link Driver} interface and is automatically registered with {@link
 * DriverManager} during class initialization. The driver intercepts {@link #connect(String,
 * Properties)} calls, generates IAM authentication tokens, and delegates to the PostgreSQL JDBC
 * driver with the temporary token set as the password.
 *
 * <h3>Driver Registration</h3>
 *
 * <p>The driver self-registers during class loading via a static initializer block. This means no
 * explicit registration is needed, having the driver on the classpath is sufficient:
 *
 * <pre>{@code
 * // No explicit registration needed
 * Connection conn = DriverManager.getConnection(url, props);
 * }</pre>
 *
 * <p>Manual control over registration is available through {@link #register()}, {@link
 * #deregister()}, and {@link #isRegistered()} for advanced scenarios.
 *
 * <h3>Credentials Sources</h3>
 *
 * <p>The driver uses AWS credentials to generate short-lived IAM authentication tokens. Credentials
 * can be sourced from:
 *
 * <ul>
 *   <li>The default AWS profile
 *   <li>A configurable AWS profile specified via the {@code profile} connection property
 *   <li>A custom credentials provider configured through {@link AuroraDsqlCredentialsManager} for
 *       advanced use cases
 * </ul>
 *
 * <p>See {@link #connect(String, Properties)} for more details on configuring the driver and the
 * connection procedure.
 *
 * @since 1.0.0
 */
public class DSQLConnector implements java.sql.Driver {

    private static final Logger PARENT_LOGGER =
            Logger.getLogger(DSQLConnector.class.getPackage().getName());

    private static @Nullable Driver registeredDriver;

    static {
        try {
            register();
        } catch (final SQLException e) {
            throw new ExceptionInInitializerError(e);
        }
    }

    /**
     * Manually registers this driver with the JDBC {@link DriverManager}.
     *
     * <p>This method is typically not needed as the driver automatically registers itself during
     * class initialization via the static initializer block. Manual registration is only necessary
     * in scenarios where the driver has been explicitly deregistered and needs to be re-registered.
     *
     * <p><b>Thread Safety:</b> This method is not thread-safe. Callers must ensure synchronization
     * if concurrent access is possible.
     *
     * @throws IllegalStateException if the driver is already registered
     * @see #deregister()
     * @see #isRegistered()
     */
    public static void register() throws SQLException {
        if (isRegistered()) {
            throw new IllegalStateException(
                    "This driver is already registered, cannot re-register");
        }
        final DSQLConnector driver = new DSQLConnector();
        DriverManager.registerDriver(driver);
        registeredDriver = driver;
    }

    /**
     * Deregisters this driver from the JDBC {@link DriverManager}.
     *
     * <p>This method prevents the driver from being called by the {@link DriverManager}. After
     * deregistration, the driver will no longer be available for creating connections through
     * {@link DriverManager#getConnection(String, Properties)}.
     *
     * <p>Deregistration is rarely needed in typical applications but may be useful in advanced use
     * cases.
     *
     * <p><b>Thread Safety:</b> This method is not thread-safe. Callers must ensure synchronization
     * if concurrent access is possible.
     *
     * @throws IllegalStateException if the driver has not been registered
     * @see #register()
     * @see #isRegistered()
     */
    public static void deregister() throws SQLException {
        if (registeredDriver == null) {
            throw new IllegalStateException(
                    "This driver has yet to be registered, cannot deregister");
        }
        DriverManager.deregisterDriver(registeredDriver);
        registeredDriver = null;
    }

    /**
     * Checks whether this driver is currently registered with the JDBC {@link DriverManager}.
     *
     * <p><b>Thread Safety:</b> This method is not thread-safe. Callers must ensure synchronization
     * if concurrent access is possible.
     *
     * @return {@code true} if the driver is registered, {@code false} otherwise
     * @see #register()
     * @see #deregister()
     */
    public static boolean isRegistered() {
        return registeredDriver != null;
    }

    /**
     * Establishes a connection to an Aurora DSQL cluster using IAM authentication.
     *
     * <p>This method orchestrates the connection process by validating the URL, extracting
     * configuration, obtaining an IAM token, and creating the underlying PostgreSQL connection. The
     * IAM token is automatically injected as the password before delegating to the PostgreSQL JDBC
     * driver.
     *
     * <h3>Connection Flow</h3>
     *
     * <ol>
     *   <li>Validates and normalizes the JDBC URL format
     *   <li>Merges properties from URL parameters and the provided {@link Properties} object
     *   <li>Obtains {@link AwsCredentialsProvider} based on {@code profile} property or custom
     *       provider chain
     *   <li>Retrieves IAM token using AWS credentials
     *   <li>Creates PostgreSQL connection to Aurora DSQL with token as password
     * </ol>
     *
     * <h3>Connection Properties</h3>
     *
     * <p>Properties can be specified in the URL or the {@link Properties} object. URL parameters
     * take precedence over {@link Properties} object values. See {@link PropertyDefinition} for all
     * available connection properties.
     *
     * <h3>Credentials Sources</h3>
     *
     * <p>The driver resolves AWS credentials in the following order:
     *
     * <ol>
     *   <li>If the {@code profile} connection property is specified, credentials are loaded from
     *       that named profile in the AWS credentials file
     *   <li>If no {@code profile} is specified, the driver uses the credentials provider configured
     *       via {@link AuroraDsqlCredentialsManager#setProvider(AwsCredentialsProvider)}
     *   <li>By default, {@link AuroraDsqlCredentialsManager} uses the AWS SDK's default credentials
     *       provider.
     * </ol>
     *
     * <h3>Error Scenarios</h3>
     *
     * <p>Throws {@link SQLException} for:
     *
     * <ul>
     *   <li>Invalid URL format
     *   <li>Missing {@code user} property
     *   <li>AWS credential resolution failures
     *   <li>IAM token generation failures
     *   <li>PostgreSQL connection failures
     * </ul>
     *
     * @param url the JDBC URL in format {@code
     *     jdbc:aws-dsql:postgresql://cluster.dsql.region.on.aws} with optional database and
     *     property definitions
     * @param info connection properties which augment those provided in the URL
     * @return a {@link Connection} to the Aurora DSQL cluster
     * @throws SQLException if connection cannot be established due to invalid input, credential
     *     issues, or connection failures
     * @see PropertyDefinition
     */
    @Override
    public Connection connect(final String url, final Properties info) throws SQLException {
        // Parse properties from URL if provided
        final Properties props = PropertyUtils.copyProperties(info);
        try {
            // Validate and construct URL if needed
            final String validatedUrl = ConnUrlParser.convertToFullJdbcUrl(url, props);

            // Parse properties from URL and merge with provided properties
            ConnUrlParser.parsePropertiesFromUrl(validatedUrl, props);

            final String user = PropertyDefinition.USER.getOrDefault(props);
            if (user == null) {
                throw new SQLException("Missing user parameter");
            }

            // Get host from URL
            final String host = ConnUrlParser.extractHostFromUrl(validatedUrl);

            // Get region from host or from properties
            final String regionStr = determineRegion(host, props);
            final Region regionObj = Region.of(regionStr.toLowerCase());

            // Get token duration from properties
            final long tokenDuration = PropertyDefinition.TOKEN_DURATION.getLong(props);
            final Duration tokenDurationInSecond =
                    tokenDuration == 0 ? null : Duration.ofSeconds(tokenDuration);

            // Get authentication token using static TokenManager
            final String authToken =
                    TokenManager.getToken(
                            host,
                            regionObj,
                            user,
                            getCredentialsProvider(props),
                            tokenDurationInSecond);
            props.setProperty("password", authToken);

            // Use ConnWrapper to create PostgreSQL connection with auth token
            final ConnWrapper connWrapper = new ConnWrapper(validatedUrl, props);
            return connWrapper.makeConnection();

        } catch (URISyntaxException e) {
            throw new SQLException("Invalid Aurora DSQL hostname format", e);
        }
    }

    /**
     * Checks if this driver can handle the specified JDBC URL.
     *
     * <p>Returns {@code true} for URLs with the {@code jdbc:aws-dsql:postgresql://} prefix. Called
     * by {@link DriverManager} to select the appropriate driver.
     *
     * @param url the JDBC URL to check
     * @return {@code true} if URL can be handled by this driver implementation
     */
    @Override
    public boolean acceptsURL(final String url) {
        return ConnUrlParser.isDsqlUrl(url);
    }

    private String determineRegion(final String host, final Properties props) throws SQLException {
        // First check if region is provided in properties
        final String regionFromProps = PropertyDefinition.REGION.get(props);
        if (regionFromProps != null) {
            return regionFromProps;
        }

        // If not in properties, extract region from host
        return ConnUrlParser.extractRegionFromHost(host);
    }

    private AwsCredentialsProvider getCredentialsProvider(final Properties props) {
        final String profile = PropertyDefinition.PROFILE.get(props);
        if (profile != null) {
            return ProfileCredentialsProvider.create(profile);
        }
        return AuroraDsqlCredentialsManager.getProvider();
    }

    /**
     * {@inheritDoc}
     *
     * @see PropertyDefinition
     */
    @Override
    @Nonnull
    public DriverPropertyInfo[] getPropertyInfo(final String url, final Properties info)
            throws SQLException {
        final Properties copy = new Properties(info);
        try {
            if (url != null) {
                ConnUrlParser.parsePropertiesFromUrl(url, copy);
            }
        } catch (URISyntaxException e) {
            throw new SQLException("Invalid URL format: " + e.getMessage(), e);
        }

        final Collection<AuroraDsqlProperty> knownProperties =
                PropertyDefinition.getAllProperties();
        final DriverPropertyInfo[] props = new DriverPropertyInfo[knownProperties.size()];
        int i = 0;
        for (final AuroraDsqlProperty prop : knownProperties) {
            props[i++] = prop.toDriverPropertyInfo(copy);
        }

        return props;
    }

    @Override
    public int getMajorVersion() {
        return Version.MAJOR;
    }

    @Override
    public int getMinorVersion() {
        return Version.MINOR;
    }

    /**
     * Provides the patch version of this driver.
     *
     * @return the patch version number
     */
    public int getPatchVersion() {
        return Version.PATCH;
    }

    /**
     * Provides the full version string of this driver (e.g., "1.2.3" or "1.2.3-SNAPSHOT").
     *
     * @return the full version string
     */
    public String getVersion() {
        return Version.FULL;
    }

    /**
     * Indicates whether this driver is JDBC compliant.
     *
     * <p>This driver returns {@code false} because it delegates to the PostgreSQL JDBC driver,
     * which is not fully JDBC compliant. The driver provides full functionality for Aurora DSQL
     * connections but does not guarantee complete JDBC specification compliance.
     *
     * @return {@code false} to indicate this driver is not fully JDBC compliant
     */
    @Override
    public boolean jdbcCompliant() {
        return false;
    }

    @SuppressFBWarnings
    @Override
    public Logger getParentLogger() {
        return PARENT_LOGGER;
    }
}
