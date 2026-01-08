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

import java.lang.reflect.Modifier;
import java.util.Arrays;
import java.util.Collection;
import java.util.Map;
import java.util.Properties;
import java.util.concurrent.ConcurrentHashMap;
import java.util.logging.Level;
import java.util.logging.Logger;

/**
 * Defines all supported connection properties for the Aurora DSQL JDBC driver.
 *
 * <p>This class provides constants for all connection properties recognized by the driver when
 * establishing a connection to Aurora DSQL. These properties configure driver behavior such as
 * credential selection and token lifetime. Properties can be set in the JDBC URL as query
 * parameters or in a {@link Properties} object passed to {@link DSQLConnector#connect(String,
 * Properties)}.
 *
 * <h3>Usage Examples</h3>
 *
 * <p>Properties in URL:
 *
 * <pre>{@code
 * String url = "jdbc:aws-dsql:postgresql://cluster.dsql.us-east-1.on.aws/postgres"
 *     + "?user=admin&profile=production&token-duration-secs=30";
 * Connection conn = DriverManager.getConnection(url);
 * }</pre>
 *
 * <p>Properties in {@link Properties} object:
 *
 * <pre>{@code
 * Properties props = new Properties();
 * props.setProperty("user", "admin");
 * props.setProperty("profile", "production");
 * props.setProperty("token-duration-secs", "30");
 *
 * String url = "jdbc:aws-dsql:postgresql://cluster.dsql.us-east-1.on.aws/postgres";
 * Connection conn = DriverManager.getConnection(url, props);
 * }</pre>
 *
 * <h3>Property Precedence</h3>
 *
 * <p>When properties are specified in both the URL and the {@link Properties} object, URL
 * parameters take precedence.
 *
 * @see DSQLConnector#connect(String, Properties)
 * @since 1.0.0
 */
public final class PropertyDefinition {

    private PropertyDefinition() {
        throw new UnsupportedOperationException(
                "This is a utility class and cannot be instantiated");
    }

    private static final Logger LOGGER = Logger.getLogger(PropertyDefinition.class.getName());

    /**
     * Database user for authentication.
     *
     * <p>This property is required for all connections.
     *
     * <p>The database role must be associated with an IAM role with sufficient IAM permissions to
     * connect to the Aurora DSQL cluster.
     *
     * @see <a
     *     href="https://docs.aws.amazon.com/aurora-dsql/latest/userguide/using-database-and-iam-roles.html">Using
     *     database roles and IAM authentication</a>
     */
    public static final AuroraDsqlProperty USER =
            new AuroraDsqlProperty("user", null, "The user to connect with Aurora DSQL");

    /**
     * Specifies the AWS credentials profile name to use for authentication.
     *
     * <p>This property is optional. If not specified, the driver uses the credentials provider
     * configured through {@link AuroraDsqlCredentialsManager} or the default credential chain.
     *
     * @see AuroraDsqlCredentialsManager
     */
    public static final AuroraDsqlProperty PROFILE =
            new AuroraDsqlProperty(
                    "profile", null, "The profile to be used for Aurora DSQL connections");

    /**
     * Specifies the AWS region for the connection.
     *
     * <p>This property is optional when the region can be parsed from the cluster hostname.
     */
    public static final AuroraDsqlProperty REGION =
            new AuroraDsqlProperty("region", null, "The AWS region for Aurora DSQL connections");

    /**
     * Specifies the validity duration for generated IAM authentication tokens in seconds.
     *
     * <p>This property is optional. If not specified, the AWS SDK default duration is used.
     */
    public static final AuroraDsqlProperty TOKEN_DURATION =
            new AuroraDsqlProperty(
                    "token-duration-secs", null, "The duration in seconds for cached tokens");

    /**
     * Specifies the database name within the Aurora DSQL cluster.
     *
     * <p>This property is optional. If not specified, the default {@code postgres} database is
     * used.
     */
    public static final AuroraDsqlProperty DATABASE =
            new AuroraDsqlProperty(
                    "database", "postgres", "The database name to connect to (default: postgres)");

    /**
     * Specifies the application name for connection tracking.
     *
     * <p>This property is optional. If set to a value without a '/' character, it will be prepended
     * to the connector identifier (e.g., "hibernate" becomes "hibernate:aurora-dsql-jdbc/1.0.0").
     * If the value contains a '/', it will be ignored and the default connector identifier will be
     * used.
     *
     * <p>This is primarily used by ORM frameworks to identify themselves in connection metrics.
     */
    public static final AuroraDsqlProperty APPLICATION_NAME =
            new AuroraDsqlProperty(
                    "ApplicationName",
                    null,
                    "Application name for connection tracking. ORM frameworks can set this to "
                            + "identify themselves (e.g., 'hibernate'). Values containing '/' are ignored.");

    private static final Map<String, AuroraDsqlProperty> PROPS_BY_NAME = new ConcurrentHashMap<>();

    static {
        registerProperties();
    }

    /**
     * Retrieves all defined connection properties.
     *
     * @return an unmodifiable {@link Collection} of all {@link AuroraDsqlProperty} instances
     *     defined by this class.
     */
    public static Collection<AuroraDsqlProperty> getAllProperties() {
        return PROPS_BY_NAME.values();
    }

    private static void registerProperties() {
        Arrays.stream(PropertyDefinition.class.getDeclaredFields())
                .filter(
                        f ->
                                f.getType() == AuroraDsqlProperty.class
                                        && Modifier.isPublic(f.getModifiers())
                                        && Modifier.isStatic(f.getModifiers()))
                .forEach(
                        f -> {
                            AuroraDsqlProperty prop = null;
                            try {
                                prop = (AuroraDsqlProperty) f.get(AuroraDsqlProperty.class);
                            } catch (final IllegalArgumentException | IllegalAccessException ex) {
                                LOGGER.log(
                                        Level.WARNING,
                                        "An error has occurred while retrieving a property",
                                        ex);
                            }

                            if (prop != null) {
                                PROPS_BY_NAME.put(prop.name, prop);
                            }
                        });
    }
}
