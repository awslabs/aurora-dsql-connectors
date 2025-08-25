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

import javax.annotation.Nonnull;
import java.net.URISyntaxException;
import java.sql.Connection;
import java.sql.DriverManager;
import java.sql.DriverPropertyInfo;
import java.sql.SQLException;
import java.sql.Driver;
import java.time.Duration;
import java.util.Collection;
import java.util.Collections;
import java.util.List;
import java.util.Properties;
import java.util.logging.Logger;
import javax.annotation.Nullable;

import edu.umd.cs.findbugs.annotations.SuppressFBWarnings;
import software.amazon.awssdk.regions.Region;
import software.amazon.awssdk.auth.credentials.AwsCredentialsProvider;
import software.amazon.awssdk.auth.credentials.ProfileCredentialsProvider;

/**
 * DSQLConnector is a package that allows Aurora DSQL developers to access
 * Aurora DSQL cluster while using this interface
 */
public class DSQLConnector implements java.sql.Driver {

    private static final Logger PARENT_LOGGER = Logger.getLogger("software.amazon.dsql.jdbc");
    private static final Logger LOGGER = Logger.getLogger("com.amazon.jdbc.DSQLConnector");
    private static @Nullable Driver registeredDriver;

    static {
        try {
            register();
        } catch (final SQLException e) {
            throw new ExceptionInInitializerError(e);
        }
    }

    public static void register() throws SQLException {
        if (isRegistered()) {
            throw new IllegalStateException("This driver is already registered, cannot re-register");
        }
        final DSQLConnector driver = new DSQLConnector();
        DriverManager.registerDriver(driver);
        registeredDriver = driver;
    }

    public static void deregister() throws SQLException {
        if (registeredDriver == null) {
            throw new IllegalStateException("This driver has yet to be registered, cannot deregister");
        }
        DriverManager.deregisterDriver(registeredDriver);
        registeredDriver = null;
    }

    public static boolean isRegistered() {
        return registeredDriver != null;
    }

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
            final Duration tokenDurationInSecond =  tokenDuration == 0 ? null : Duration.ofSeconds(tokenDuration);

            // Get authentication token using static TokenManager
            final String authToken = TokenManager.getToken(
                host, 
                regionObj, 
                user, 
                getCredentialsProvider(props),
                tokenDurationInSecond
            );
            props.setProperty("password", authToken);

            // Use ConnWrapper to create PostgreSQL connection with auth token
            final ConnWrapper connWrapper = new ConnWrapper(validatedUrl, props);
            return connWrapper.makeConnection();

        } catch (URISyntaxException e) {
            throw new SQLException("Invalid Aurora DSQL hostname format", e);
        }
    }

    @Override
    public boolean acceptsURL(final String url) throws SQLException {
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

    @Override
    @Nonnull
    public DriverPropertyInfo[] getPropertyInfo(final String url, final Properties info) throws SQLException {
        final Properties copy = new Properties(info);
        try {
            if (url != null) {
                ConnUrlParser.parsePropertiesFromUrl(url, copy);
            }
        } catch (URISyntaxException e) {
            throw new SQLException("Invalid URL format: " + e.getMessage(), e);
        }

        final Collection<AuroraDsqlProperty> knownProperties = PropertyDefinition.getAllProperties();
        final DriverPropertyInfo[] props = new DriverPropertyInfo[knownProperties.size()];
        int i = 0;
        for (final AuroraDsqlProperty prop : knownProperties) {
            props[i++] = prop.toDriverPropertyInfo(copy);
        }

        return props;
    }

    @Override
    public int getMajorVersion() {
        return 1;
    }

    @Override
    public int getMinorVersion() {
        return 0;
    }

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

