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

import java.net.URI;
import java.net.URISyntaxException;
import java.sql.SQLException;
import java.util.Properties;
import javax.annotation.Nonnull;
import software.amazon.awssdk.utils.StringUtils;

public final class ConnUrlParser {

    private static final LazyLogger LOGGER = new LazyLogger(ConnUrlParser.class.getName());
    private static final String JDBC_PREFIX = "jdbc:postgresql://";
    private static final String CONNECTOR_POSTGRESQL_PREFIX = "jdbc:aws-dsql:postgresql://";

    private ConnUrlParser() {
        throw new UnsupportedOperationException(
                "This is a utility class and cannot be instantiated");
    }

    /**
     * Parse properties from URL and add them to the provided Properties object.
     *
     * @param url URL to parse
     * @param props Properties object to add parsed properties to
     * @throws URISyntaxException if the URL is invalid
     */
    public static void parsePropertiesFromUrl(final String url, @Nonnull final Properties props)
            throws URISyntaxException {
        if (url == null) {
            return;
        }

        final String cleanUrl = url.substring(5);
        final URI connUrl = new URI(cleanUrl);

        // Extract database name from path if present and not already set in properties
        if (props.getProperty("database") == null
                && connUrl.getPath() != null
                && !connUrl.getPath().isEmpty()) {
            final String database = connUrl.getPath().substring(1); // Remove leading '/'
            props.setProperty("database", database);
        }

        final String params = connUrl.getQuery();
        if (params == null) {
            return;
        }

        final String[] listOfParameters = params.split("&");
        for (final String param : listOfParameters) {
            final String[] keyValPair = param.split("=");

            if (keyValPair.length == 1) {
                props.setProperty(keyValPair[0], "");
                continue;
            }

            props.setProperty(keyValPair[0], keyValPair[1]);
        }
    }

    /**
     * Converts a DSQL URL to a full JDBC URL with database and parameters.
     *
     * @param url DSQL URL (with or without jdbc: prefix, with or without database)
     * @param props Properties containing connection parameters
     * @return Full JDBC URL with database and parameters
     * @throws SQLException if URL conversion fails
     */
    @Nonnull
    public static String convertToFullJdbcUrl(final String url, @Nonnull final Properties props)
            throws SQLException {
        if (url == null) {
            throw new SQLException("URL is null");
        }

        if (!isDsqlUrl(url)) {
            throw new SQLException("URL must be a valid Aurora DSQL URL");
        }

        try {
            String cleanUrl = url.substring(CONNECTOR_POSTGRESQL_PREFIX.length());

            // Handle jdbc:aws-dsql:postgresql:// prefix

            // Parse as postgresql:// URL
            final String uriString = "postgresql://" + cleanUrl;
            final URI uri = new URI(uriString);
            final String host = uri.getHost();
            final int port = uri.getPort();

            if (host == null) {
                throw new SQLException("Invalid URL format: no host found");
            }

            // Get database from URL or properties
            String database = extractDatabaseFromUrl(url);
            if (database == null || database.isEmpty()) {
                database = PropertyDefinition.DATABASE.getOrDefault(props);
            }

            // Build the base JDBC URL
            StringBuilder jdbcUrl = new StringBuilder();
            jdbcUrl.append(JDBC_PREFIX).append(host);
            if (port != -1) {
                jdbcUrl.append(":").append(port);
            }
            jdbcUrl.append("/").append(database);

            // Preserve existing query parameters from URL
            String existingParams = uri.getQuery();
            if (existingParams != null && !existingParams.isEmpty()) {
                jdbcUrl.append("?").append(existingParams);
            }

            return jdbcUrl.toString();

        } catch (URISyntaxException e) {
            throw new SQLException("Invalid URL format", e);
        }
    }

    /**
     * Checks if the URL is a valid Aurora DSQL URL. Only accepts jdbc:aws-dsql:postgresql://
     * connector prefix with valid DSQL hostname.
     *
     * @param url URL to check
     * @return true if the URL is a valid Aurora DSQL URL
     */
    public static boolean isDsqlUrl(final String url) {
        return !StringUtils.isEmpty(url) && url.startsWith(CONNECTOR_POSTGRESQL_PREFIX);
    }

    /**
     * Extracts the hostname from a JDBC URL.
     *
     * @param url JDBC URL
     * @return Hostname or null if not found
     * @throws SQLException if URL is invalid
     */
    @Nonnull
    public static String extractHostFromUrl(final String url) throws SQLException {
        if (url == null) {
            throw new SQLException("URL is null");
        }

        try {
            final String cleanUrl = url.substring(5);
            final URI uri = new URI(cleanUrl);

            if (uri.getHost().isEmpty()) {
                throw new SQLException("Invalid URL format");
            }
            return uri.getHost();
        } catch (Exception e) {
            throw new SQLException("Invalid URL format", e);
        }
    }

    /**
     * Extracts the region from a hostname.
     *
     * @param host Hostname
     * @return Region extracted from the hostname
     * @throws SQLException if region cannot be determined from the host
     */
    public static String extractRegionFromHost(final String host) throws SQLException {
        if (host == null) {
            throw new SQLException("Cannot determine region: Host is null");
        }

        final String[] hostParts = host.split("\\.");
        if (hostParts.length >= 3) {
            return hostParts[2];
        }

        throw new SQLException("Cannot determine region from host: " + host);
    }

    /**
     * Extracts the database name from a JDBC URL.
     *
     * @param url JDBC URL
     * @return Database name or null if not found
     */
    public static String extractDatabaseFromUrl(final String url) {
        if (url == null) {
            return null;
        }

        try {
            String cleanUrl = url;

            // Remove jdbc: prefix if present
            if (url.startsWith("jdbc:")) {
                cleanUrl = url.substring(5);
            }

            // Add a temporary scheme if the URL doesn't have one (for proper URI parsing)
            String uriString = cleanUrl;
            if (!cleanUrl.startsWith("postgresql://")) {
                uriString = "postgresql://" + cleanUrl;
            }

            // Parse as URI
            final URI uri = new URI(uriString);

            if (uri.getPath() != null && !uri.getPath().isEmpty() && !uri.getPath().equals("/")) {
                String pathPart = uri.getPath();
                if (pathPart.startsWith("/")) {
                    pathPart = pathPart.substring(1); // Remove leading '/'
                }

                // Return the database name (everything before any additional path segments)
                int slashIndex = pathPart.indexOf('/');
                if (slashIndex != -1) {
                    pathPart = pathPart.substring(0, slashIndex);
                }

                return pathPart.isEmpty() ? null : pathPart;
            }
        } catch (Exception e) {
            LOGGER.warning("Could not parse database from URL: " + e.getMessage());
        }

        return null;
    }
}
