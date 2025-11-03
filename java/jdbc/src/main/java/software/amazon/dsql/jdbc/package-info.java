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

/**
 * Aurora DSQL Connector for JDBC - IAM authentication plugin for PostgreSQL JDBC connections to
 * Amazon Aurora DSQL.
 *
 * <h2>Overview</h2>
 *
 * <p>The Aurora DSQL Connector for JDBC extends the PostgreSQL JDBC driver to enable IAM-based
 * authentication for Amazon Aurora DSQL clusters. It automatically generates and manages
 * time-limited IAM authentication tokens, allowing applications to connect to Aurora DSQL using
 * standard JDBC patterns without manual token handling. The connector acts as an authentication
 * layer on top of the PostgreSQL JDBC driver, providing seamless integration with AWS credential
 * providers.
 *
 * <p>Key features include automatic IAM token generation, intelligent token caching with refresh
 * logic, support for multiple AWS credential providers, and full compatibility with JDBC connection
 * pooling libraries such as HikariCP.
 *
 * <h2>Basic Usage</h2>
 *
 * <p>Connect to Aurora DSQL using the {@code jdbc:aws-dsql:postgresql://} URL prefix. The connector
 * uses sensible defaults for optional properties such as profile (default credential provider
 * chain) and token duration (AWS SDK default).
 *
 * <pre>{@code
 * Properties props = new Properties();
 * props.setProperty("user", "admin");
 *
 * // Optional: specify AWS profile for credentials
 * props.setProperty("profile", "myprofile");
 *
 * // Optional: customize token duration in seconds
 * props.setProperty("token-duration-secs", "30");
 *
 * String url = "jdbc:aws-dsql:postgresql://your-cluster.dsql.us-east-1.on.aws";
 * Connection conn = DriverManager.getConnection(url, props);
 * }</pre>
 *
 * <h2>Key Concepts</h2>
 *
 * <h3>IAM Authentication</h3>
 *
 * <p>Aurora DSQL requires IAM-based authentication using time-limited tokens. The connector
 * automatically generates these tokens using AWS credentials from the default credential provider
 * chain, a specified profile, or a custom credentials provider. Tokens are generated on-demand
 * during connection establishment and include the cluster hostname, region, and user information.
 *
 * <h3>Thread Safety</h3>
 *
 * <p>The connector is designed for concurrent use. Thread-safe caching mechanisms are used to
 * ensure multiple threads can safely request tokens simultaneously. Connection establishment
 * through {@link software.amazon.dsql.jdbc.DSQLConnector} is thread-safe and can be called
 * concurrently from multiple threads.
 *
 * <h2>Configuration Properties</h2>
 *
 * <p>The connector supports the following configuration properties:
 *
 * <ul>
 *   <li>{@code user} - Database user for authentication (required)
 *   <li>{@code profile} - AWS credentials profile name (optional)
 *   <li>{@code token-duration-secs} - Token validity duration in seconds (optional)
 *   <li>{@code region} - AWS region override (optional, parsed from URL by default)
 * </ul>
 *
 * <p>Properties can be specified in the URL or a {@link java.util.Properties} object.
 *
 * <p>The database name is specified in the URL path (e.g., {@code /postgres}). If not specified in
 * the URL, it defaults to {@code postgres}.
 *
 * <h2>Classes</h2>
 *
 * <ul>
 *   <li>{@link software.amazon.dsql.jdbc.DSQLConnector} - Main entry point implementing JDBC
 *       interface
 *   <li>{@link software.amazon.dsql.jdbc.AuroraDsqlCredentialsManager} - Manages AWS credentials
 *       provider configuration
 *   <li>{@link software.amazon.dsql.jdbc.PropertyDefinition} - Property constants (USER, PROFILE,
 *       REGION, etc.)
 *   <li>{@link software.amazon.dsql.jdbc.AuroraDsqlProperty} - Provides a representation of
 *       configuration property metadata
 * </ul>
 *
 * <p>For comprehensive setup instructions, installation details, and additional examples, see the
 * <a href="https://github.com/awslabs/aurora-dsql-jdbc-connector/blob/main/README.md">README</a>.
 *
 * @see software.amazon.dsql.jdbc.DSQLConnector
 * @see software.amazon.dsql.jdbc.AuroraDsqlCredentialsManager
 * @see software.amazon.dsql.jdbc.PropertyDefinition
 * @see software.amazon.dsql.jdbc.AuroraDsqlProperty
 * @since 1.0.0
 */
package software.amazon.dsql.jdbc;
