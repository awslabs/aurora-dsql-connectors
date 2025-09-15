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

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.net.URISyntaxException;
import java.sql.SQLException;
import java.util.Properties;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

class ConnUrlParserTest {

    private Properties properties;

    @BeforeEach
    void setUp() {
        properties = new Properties();
    }

    @Test
    void testParsePropertiesFromValidUrl() throws URISyntaxException {
        // Arrange
        String url =
                "jdbc:postgresql://test-cluster.dsql.us-east-1.on.aws/postgres?user=admin&token-duration-secs=3600";

        // Act
        ConnUrlParser.parsePropertiesFromUrl(url, properties);

        // Assert
        assertEquals("admin", properties.getProperty("user"));
        assertEquals("3600", properties.getProperty("token-duration-secs"));
        assertEquals("postgres", properties.getProperty("database"));
    }

    @Test
    void testParsePropertiesFromUrlWithCustomDatabase() throws URISyntaxException {
        // Arrange
        String url = "jdbc:postgresql://test-cluster.dsql.us-east-1.on.aws/custom_db?user=admin";

        // Act
        ConnUrlParser.parsePropertiesFromUrl(url, properties);

        // Assert
        assertEquals("admin", properties.getProperty("user"));
        assertEquals("custom_db", properties.getProperty("database"));
    }

    @Test
    void testParsePropertiesFromUrlWithDatabaseInProperties() throws URISyntaxException {
        // Arrange
        String url = "jdbc:postgresql://test-cluster.dsql.us-east-1.on.aws/postgres?user=admin";
        properties.setProperty("database", "custom_db");

        // Act
        ConnUrlParser.parsePropertiesFromUrl(url, properties);

        // Assert
        assertEquals("admin", properties.getProperty("user"));
        assertEquals(
                "custom_db", properties.getProperty("database")); // Should keep the property value
    }

    @Test
    void testParsePropertiesFromUrlWithMultipleParameters() throws URISyntaxException {
        // Arrange
        String url =
                "jdbc:postgresql://cluster.dsql.us-west-2.on.aws/mydb?user=testuser&token-duration-secs=7200&profile=myprofile&ssl=true";

        // Act
        ConnUrlParser.parsePropertiesFromUrl(url, properties);

        // Assert
        assertEquals("testuser", properties.getProperty("user"));
        assertEquals("7200", properties.getProperty("token-duration-secs"));
        assertEquals("myprofile", properties.getProperty("profile"));
        assertEquals("true", properties.getProperty("ssl"));
        assertEquals("mydb", properties.getProperty("database"));
    }

    @Test
    void testParsePropertiesFromUrlWithClusterIdAndRegion() throws URISyntaxException {
        // Arrange
        String url =
                "jdbc:postgresql://dummy.host/postgres?user=admin&cluster-id=my-cluster&region=eu-west-1";

        // Act
        ConnUrlParser.parsePropertiesFromUrl(url, properties);

        // Assert
        assertEquals("admin", properties.getProperty("user"));
        assertEquals("my-cluster", properties.getProperty("cluster-id"));
        assertEquals("eu-west-1", properties.getProperty("region"));
        assertEquals("postgres", properties.getProperty("database"));
    }

    @Test
    void testParsePropertiesFromUrlWithNoParameters() throws URISyntaxException {
        // Arrange
        String url = "jdbc:postgresql://cluster.dsql.us-east-1.on.aws/postgres";

        // Act
        ConnUrlParser.parsePropertiesFromUrl(url, properties);

        // Assert
        assertEquals("postgres", properties.getProperty("database"));
        assertEquals(1, properties.size()); // Only database property should be set
    }

    @Test
    void testParsePropertiesFromUrlWithEmptyParameters() throws URISyntaxException {
        // Arrange
        String url = "jdbc:postgresql://cluster.dsql.us-east-1.on.aws/postgres?";

        // Act
        ConnUrlParser.parsePropertiesFromUrl(url, properties);

        // Assert
        assertEquals("postgres", properties.getProperty("database"));
        assertTrue(properties.size() <= 2); // Could be 1 or 2 depending on implementation
    }

    @Test
    void testParsePropertiesFromUrlWithParameterWithoutValue() throws URISyntaxException {
        // Arrange
        String url = "jdbc:postgresql://cluster.dsql.us-east-1.on.aws/postgres?ssl&user=admin";

        // Act
        ConnUrlParser.parsePropertiesFromUrl(url, properties);

        // Assert
        assertEquals("", properties.getProperty("ssl")); // Should be empty string
        assertEquals("admin", properties.getProperty("user"));
        assertEquals("postgres", properties.getProperty("database"));
    }

    @Test
    void testParsePropertiesFromUrlWithEmptyParameterValue() throws URISyntaxException {
        // Arrange
        String url =
                "jdbc:postgresql://cluster.dsql.us-east-1.on.aws/postgres?user=&token-duration-secs=3600";

        // Act
        ConnUrlParser.parsePropertiesFromUrl(url, properties);

        // Assert
        assertEquals("", properties.getProperty("user"));
        assertEquals("3600", properties.getProperty("token-duration-secs"));
        assertEquals("postgres", properties.getProperty("database"));
    }

    @Test
    void testParsePropertiesFromInvalidUrl() {
        // Arrange - URL that will cause URI parsing issues
        String url = "jdbc:postgresql://[invalid-host-format]/database";

        // Act & Assert
        assertThrows(
                URISyntaxException.class,
                () -> ConnUrlParser.parsePropertiesFromUrl(url, properties));
    }

    @Test
    void testParsePropertiesFromNullUrl() {
        // Act
        assertDoesNotThrow(() -> ConnUrlParser.parsePropertiesFromUrl(null, properties));
    }

    @Test
    void testConstructorThrowsException() {
        // Act & Assert
        Exception exception =
                assertThrows(
                        Exception.class,
                        () -> {
                            // Use reflection to access private constructor
                            java.lang.reflect.Constructor<ConnUrlParser> constructor =
                                    ConnUrlParser.class.getDeclaredConstructor();
                            constructor.setAccessible(true);
                            constructor.newInstance();
                        });

        // The actual exception will be wrapped in InvocationTargetException
        assertTrue(exception.getCause() instanceof UnsupportedOperationException);
    }

    @Test
    void testIsDsqlUrl() {
        // Valid DSQL URLs
        assertTrue(
                ConnUrlParser.isDsqlUrl(
                        "jdbc:aws-dsql:postgresql://cluster-name.dsql.eu-west-1.on.aws/mydb"));
        assertFalse(
                ConnUrlParser.isDsqlUrl(
                        "jdbc:postgresql://my-cluster.rds.us-east-1.amazonaws.com/postgres"));
        assertFalse(ConnUrlParser.isDsqlUrl(null));
    }

    @Test
    void testExtractDatabaseFromUrl() {
        // Test with valid URL
        assertEquals(
                "postgres",
                ConnUrlParser.extractDatabaseFromUrl(
                        "jdbc:postgresql://my-cluster.dsql.us-east-1.on.aws/postgres"));

        // Test with custom database
        assertEquals(
                "mydb",
                ConnUrlParser.extractDatabaseFromUrl(
                        "jdbc:postgresql://my-cluster.dsql.us-east-1.on.aws/mydb"));

        // Test with null URL
        assertNull(ConnUrlParser.extractDatabaseFromUrl(null));

        // Test with invalid URL
        assertNull(ConnUrlParser.extractDatabaseFromUrl("invalid-url"));
    }

    @Test
    void testExtractHostFromUrl() throws SQLException {
        // Test with valid URL
        assertEquals(
                "my-cluster.dsql.us-east-1.on.aws",
                ConnUrlParser.extractHostFromUrl(
                        "jdbc:postgresql://my-cluster.dsql.us-east-1.on.aws/postgres"));

        assertEquals(
                "my-cluster.dsql-gamma.us-east-1.on.aws",
                ConnUrlParser.extractHostFromUrl(
                        "jdbc:postgresql://my-cluster.dsql-gamma.us-east-1.on.aws/postgres"));

        // Test with null URL
        assertThrows(SQLException.class, () -> ConnUrlParser.extractHostFromUrl(null));

        // Test with invalid URL
        assertThrows(SQLException.class, () -> ConnUrlParser.extractHostFromUrl("invalid-url"));
    }

    @Test
    void testExtractRegionFromHost() throws SQLException {
        // Test with valid host
        assertEquals(
                "us-east-1",
                ConnUrlParser.extractRegionFromHost("my-cluster.dsql.us-east-1.on.aws"));

        // Test with null host
        assertThrows(SQLException.class, () -> ConnUrlParser.extractRegionFromHost(null));

        // Test with invalid host
        assertThrows(SQLException.class, () -> ConnUrlParser.extractRegionFromHost("invalid-host"));
    }

    @Test
    void testconvertToFullJdbcUrlWithValidDsqlUrl() throws SQLException {
        // Arrange
        String url = "jdbc:aws-dsql:postgresql://my-cluster.dsql.us-east-1.on.aws/postgres";
        Properties props = new Properties();

        // Act
        String result = ConnUrlParser.convertToFullJdbcUrl(url, props);

        // Assert
        assertEquals("jdbc:postgresql://my-cluster.dsql.us-east-1.on.aws/postgres", result);
    }
}
