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

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.mockConstruction;
import static org.mockito.Mockito.mockStatic;
import static org.mockito.Mockito.when;

import java.net.URISyntaxException;
import java.sql.Connection;
import java.sql.DriverPropertyInfo;
import java.sql.SQLException;
import java.time.Duration;
import java.util.Arrays;
import java.util.Collections;
import java.util.Properties;
import java.util.logging.Logger;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.MockedConstruction;
import org.mockito.MockedStatic;
import org.mockito.junit.jupiter.MockitoExtension;
import software.amazon.awssdk.auth.credentials.AwsCredentialsProvider;
import software.amazon.awssdk.auth.credentials.ProfileCredentialsProvider;
import software.amazon.awssdk.regions.Region;

@ExtendWith(MockitoExtension.class)
class DSQLConnectorTest {

    private DSQLConnector driver;
    private MockedStatic<PropertyUtils> propertyUtilsMock;
    private MockedStatic<AuroraDsqlCredentialsManager> credentialsManagerMock;
    private MockedStatic<ProfileCredentialsProvider> profileCredentialsProviderMock;
    private MockedStatic<TokenManager> tokenManagerMockedStatic;

    private boolean wasRegisteredBefore; // Track initial state

    @BeforeEach
    void setUp() throws SQLException {
        // Store the initial registration state
        wasRegisteredBefore = DSQLConnector.isRegistered();

        driver = new DSQLConnector();

        // Setup static mocks
        propertyUtilsMock = mockStatic(PropertyUtils.class);
        credentialsManagerMock = mockStatic(AuroraDsqlCredentialsManager.class);
        profileCredentialsProviderMock = mockStatic(ProfileCredentialsProvider.class);
        tokenManagerMockedStatic = mockStatic(TokenManager.class);

        // Default stubbing for TokenManager
        tokenManagerMockedStatic
                .when(
                        () ->
                                TokenManager.getToken(
                                        anyString(),
                                        any(Region.class),
                                        anyString(),
                                        any(AwsCredentialsProvider.class),
                                        any()))
                .thenReturn("mock-token");
    }

    @AfterEach
    void tearDown() throws SQLException {
        // Close all static mocks
        propertyUtilsMock.close();
        credentialsManagerMock.close();
        profileCredentialsProviderMock.close();
        tokenManagerMockedStatic.close();

        // Restore the initial registration state
        try {
            if (wasRegisteredBefore && !DSQLConnector.isRegistered()) {
                DSQLConnector.register();
            } else if (!wasRegisteredBefore && DSQLConnector.isRegistered()) {
                DSQLConnector.deregister();
            }
        } catch (Exception e) {
            // Log but don't fail the test cleanup
            System.err.println(
                    "Warning: Could not restore driver registration state: " + e.getMessage());
        }
    }

    @Test
    void testRegister_Success() throws SQLException {
        // Arrange - Ensure we start from a clean state
        if (DSQLConnector.isRegistered()) {
            DSQLConnector.deregister();
        }

        // Act
        DSQLConnector.register();

        // Assert
        assertTrue(DSQLConnector.isRegistered());
    }

    @Test
    void testRegister_AlreadyRegistered_ThrowsException() throws SQLException {
        // Arrange - Ensure driver is registered
        if (!DSQLConnector.isRegistered()) {
            DSQLConnector.register();
        }

        // Act & Assert
        IllegalStateException exception =
                assertThrows(
                        IllegalStateException.class,
                        () -> {
                            DSQLConnector.register();
                        });
        assertEquals(
                "This driver is already registered, cannot re-register", exception.getMessage());
    }

    @Test
    void testDeregister_Success() throws SQLException {
        // Arrange - Ensure driver is registered
        if (!DSQLConnector.isRegistered()) {
            DSQLConnector.register();
        }

        // Act
        DSQLConnector.deregister();

        // Assert
        assertFalse(DSQLConnector.isRegistered());
    }

    @Test
    void testDeregister_NotRegistered_ThrowsException() throws SQLException {
        // Arrange - Ensure driver is not registered
        if (DSQLConnector.isRegistered()) {
            DSQLConnector.deregister();
        }

        // Act & Assert
        IllegalStateException exception =
                assertThrows(
                        IllegalStateException.class,
                        () -> {
                            DSQLConnector.deregister();
                        });
        assertEquals(
                "This driver has yet to be registered, cannot deregister", exception.getMessage());
    }

    @Test
    void testAcceptsURL_AwsDsqlPostgresqlPrefix() throws SQLException {
        // Test jdbc:aws-dsql:postgresql:// format only
        assertTrue(
                driver.acceptsURL(
                        "jdbc:aws-dsql:postgresql://test-cluster.dsql.us-east-1.on.aws/testdb"));
        assertTrue(
                driver.acceptsURL(
                        "jdbc:aws-dsql:postgresql://cluster.dsql.us-west-2.on.aws/postgres?user=admin"));
    }

    @Test
    void testConnect_Success() throws SQLException, URISyntaxException {
        // Arrange
        String url = "jdbc:aws-dsql:postgresql://cluster.dsql.us-west-2.on.aws/postgres";
        Properties info = new Properties();
        info.setProperty("user", "testuser");

        Properties copiedProps = new Properties();
        copiedProps.setProperty("user", "testuser");

        // Mock static method calls
        propertyUtilsMock.when(() -> PropertyUtils.copyProperties(info)).thenReturn(copiedProps);

        AwsCredentialsProvider credentialsProvider = mock(AwsCredentialsProvider.class);
        credentialsManagerMock
                .when(() -> AuroraDsqlCredentialsManager.getProvider())
                .thenReturn(credentialsProvider);

        // Mock ConnWrapper to prevent actual network connection
        Connection mockConnection = mock(Connection.class);
        try (MockedConstruction<ConnWrapper> connWrapperMock =
                mockConstruction(
                        ConnWrapper.class,
                        (mock, context) -> {
                            when(mock.makeConnection()).thenReturn(mockConnection);
                        })) {
            // Act
            Connection result = driver.connect(url, info);

            // Assert
            assertNotNull(result);
            assertEquals(mockConnection, result);
        }
    }

    @Test
    void testConnect_MissingUser_ThrowsException() throws SQLException, URISyntaxException {
        // Arrange
        String url = "jdbc:aws-dsql:postgresql://test-cluster.dsql.us-east-1.on.aws/testdb";
        Properties info = new Properties();

        Properties copiedProps = new Properties(); // No user property

        propertyUtilsMock.when(() -> PropertyUtils.copyProperties(info)).thenReturn(copiedProps);

        // Act & Assert
        SQLException exception =
                assertThrows(
                        SQLException.class,
                        () -> {
                            driver.connect(url, info);
                        });
        assertEquals("Missing user parameter", exception.getMessage());
    }

    @Test
    void testConnect_InvalidURL_ThrowsSQLException() throws SQLException {
        // Arrange - Use a truly invalid URL that will cause parsing issues
        String url = null; // or some other invalid format
        Properties info = new Properties();
        info.setProperty("user", "testuser");

        Properties copiedProps = new Properties();
        copiedProps.setProperty("user", "testuser");

        propertyUtilsMock.when(() -> PropertyUtils.copyProperties(info)).thenReturn(copiedProps);
        // Let the actual method handle the invalid URL

        // Act & Assert
        SQLException exception =
                assertThrows(
                        SQLException.class,
                        () -> {
                            driver.connect(url, info);
                        });
        // Verify it's the expected error message
        assertTrue(
                exception.getMessage().contains("URL")
                        || exception.getMessage().contains("null")
                        || exception.getMessage().contains("Invalid"));
    }

    @Test
    void testConnect_WithProfile() throws SQLException, URISyntaxException {
        // Arrange
        String url = "jdbc:aws-dsql:postgresql://test-cluster.dsql.us-east-1.on.aws/testdb";
        Properties info = new Properties();
        info.setProperty("user", "testuser");
        info.setProperty("profile", "test-profile");

        Properties copiedProps = new Properties();
        copiedProps.setProperty("user", "testuser");
        copiedProps.setProperty("profile", "test-profile");
        copiedProps.setProperty("region", "us-west-2");

        propertyUtilsMock.when(() -> PropertyUtils.copyProperties(info)).thenReturn(copiedProps);

        ProfileCredentialsProvider profileProvider = mock(ProfileCredentialsProvider.class);
        profileCredentialsProviderMock
                .when(() -> ProfileCredentialsProvider.create("test-profile"))
                .thenReturn(profileProvider);

        tokenManagerMockedStatic
                .when(
                        () ->
                                TokenManager.getToken(
                                        anyString(),
                                        any(Region.class),
                                        anyString(),
                                        any(AwsCredentialsProvider.class),
                                        any(Duration.class)))
                .thenReturn("mock-token");

        // Mock ConnWrapper to prevent actual network connection
        Connection mockConnection = mock(Connection.class);
        try (MockedConstruction<ConnWrapper> connWrapperMock =
                mockConstruction(
                        ConnWrapper.class,
                        (mock, context) -> {
                            when(mock.makeConnection()).thenReturn(mockConnection);
                        })) {
            // Act
            Connection result = driver.connect(url, info);

            // Assert
            assertNotNull(result);
            assertEquals(mockConnection, result);
        }
    }

    @Test
    void testAcceptsURL_ValidDsqlUrls() throws SQLException {
        // Test only jdbc:aws-dsql:postgresql:// format
        assertTrue(
                driver.acceptsURL(
                        "jdbc:aws-dsql:postgresql://mycluster.dsql.us-east-1.on.aws/postgres"));
        assertTrue(
                driver.acceptsURL(
                        "jdbc:aws-dsql:postgresql://test-cluster.dsql.us-east-1.on.aws/db"));
    }

    @Test
    void testAcceptsURL_InvalidUrls() throws SQLException {
        // Test null and empty URLs - should return false
        assertFalse(driver.acceptsURL(null));
        assertFalse(driver.acceptsURL(""));

        // Test non-DSQL PostgreSQL URLs - should return false
        assertFalse(driver.acceptsURL("jdbc:postgresql://localhost:5432/postgres"));
        assertFalse(driver.acceptsURL("jdbc:postgresql://rds.amazonaws.com:5432/db"));
        assertFalse(driver.acceptsURL("jdbc:postgresql://regular-postgres.com/db"));

        // Test raw DSQL URLs (no longer supported) - should return false
        assertFalse(driver.acceptsURL("test-cluster.dsql.us-east-1.on.aws/db"));
        assertFalse(driver.acceptsURL("my-cluster.dsql.us-east-1.on.aws/postgres"));

        // Test standard PostgreSQL DSQL URLs (no longer supported) - should return false
        assertFalse(driver.acceptsURL("jdbc:postgresql://test-cluster.dsql.us-east-1.on.aws/db"));

        // Test old jdbc:aws-dsql format (no longer supported) - should return false
        assertFalse(driver.acceptsURL("jdbc:aws-dsql:test-cluster.dsql.us-east-1.on.aws/db"));

        // Test other database URLs - should return false
        assertFalse(driver.acceptsURL("jdbc:mysql://localhost:3306/db"));
        assertFalse(driver.acceptsURL("jdbc:oracle:thin:@localhost:1521:xe"));

        // Test malformed URLs - should return false
        assertFalse(driver.acceptsURL("invalid-url"));
        assertFalse(driver.acceptsURL("not-a-url"));
    }

    @Test
    void testGetPropertyInfo() throws SQLException, URISyntaxException {
        // Arrange
        String url = "jdbc:postgresql://test.dsql.us-east-1.on.aws/db?user=test";
        Properties info = new Properties();
        info.setProperty("password", "secret");

        // Mock PropertyDefinition.getAllProperties()
        AuroraDsqlProperty mockProperty1 = mock(AuroraDsqlProperty.class);
        AuroraDsqlProperty mockProperty2 = mock(AuroraDsqlProperty.class);

        DriverPropertyInfo mockPropertyInfo1 = new DriverPropertyInfo("user", "test");
        DriverPropertyInfo mockPropertyInfo2 = new DriverPropertyInfo("password", "secret");

        when(mockProperty1.toDriverPropertyInfo(any(Properties.class)))
                .thenReturn(mockPropertyInfo1);
        when(mockProperty2.toDriverPropertyInfo(any(Properties.class)))
                .thenReturn(mockPropertyInfo2);

        try (MockedStatic<PropertyDefinition> propertyDefinitionMock =
                mockStatic(PropertyDefinition.class)) {
            propertyDefinitionMock
                    .when(() -> PropertyDefinition.getAllProperties())
                    .thenReturn(Arrays.asList(mockProperty1, mockProperty2));

            // Act
            DriverPropertyInfo[] result = driver.getPropertyInfo(url, info);

            // Assert
            assertNotNull(result);
            assertEquals(2, result.length);
            assertEquals(mockPropertyInfo1, result[0]);
            assertEquals(mockPropertyInfo2, result[1]);
        }
    }

    @Test
    void testGetPropertyInfo_WithNullUrl() throws SQLException {
        // Arrange
        Properties info = new Properties();

        AuroraDsqlProperty mockProperty = mock(AuroraDsqlProperty.class);
        DriverPropertyInfo mockPropertyInfo = new DriverPropertyInfo("test", "value");
        when(mockProperty.toDriverPropertyInfo(any(Properties.class))).thenReturn(mockPropertyInfo);

        try (MockedStatic<PropertyDefinition> propertyDefinitionMock =
                mockStatic(PropertyDefinition.class)) {
            propertyDefinitionMock
                    .when(() -> PropertyDefinition.getAllProperties())
                    .thenReturn(Collections.singletonList(mockProperty));

            // Act
            DriverPropertyInfo[] result = driver.getPropertyInfo(null, info);

            // Assert
            assertNotNull(result);
            assertEquals(1, result.length);
        }
    }

    @Test
    void testGetMajorVersion() {
        assertEquals(1, driver.getMajorVersion());
    }

    @Test
    void testGetMinorVersion() {
        assertEquals(0, driver.getMinorVersion());
    }

    @Test
    void testJdbcCompliant() {
        assertFalse(driver.jdbcCompliant());
    }

    @Test
    void testGetParentLogger() {
        Logger logger = driver.getParentLogger();
        assertNotNull(logger);
        assertEquals("software.amazon.dsql.jdbc", logger.getName());
    }

    @Test
    void testConnect_WithRegionFromProperties() throws SQLException, URISyntaxException {
        // Arrange
        String url = "jdbc:aws-dsql:postgresql://test-cluster.dsql.us-east-1.on.aws/testdb";
        Properties info = new Properties();
        info.setProperty("user", "testuser");
        info.setProperty("region", "us-west-2"); // Region in properties should override hostname

        Properties copiedProps = new Properties();
        copiedProps.setProperty("user", "testuser");
        copiedProps.setProperty("region", "us-west-2");

        propertyUtilsMock.when(() -> PropertyUtils.copyProperties(info)).thenReturn(copiedProps);

        AwsCredentialsProvider credentialsProvider = mock(AwsCredentialsProvider.class);
        credentialsManagerMock
                .when(() -> AuroraDsqlCredentialsManager.getProvider())
                .thenReturn(credentialsProvider);

        // Mock ConnWrapper to prevent actual network connection
        Connection mockConnection = mock(Connection.class);
        try (MockedConstruction<ConnWrapper> connWrapperMock =
                mockConstruction(
                        ConnWrapper.class,
                        (mock, context) -> {
                            when(mock.makeConnection()).thenReturn(mockConnection);
                        })) {
            // Act
            Connection result = driver.connect(url, info);

            // Assert
            assertNotNull(result);
        }
    }

    @Test
    void testConnect_WithoutRegionInProperties_UsesHostname()
            throws SQLException, URISyntaxException {
        // Arrange
        String url = "jdbc:aws-dsql:postgresql://test-cluster.dsql.us-east-1.on.aws/testdb";
        Properties info = new Properties();
        info.setProperty("user", "testuser");
        // No region property - should extract from hostname

        Properties copiedProps = new Properties();
        copiedProps.setProperty("user", "testuser");
        // No region property

        propertyUtilsMock.when(() -> PropertyUtils.copyProperties(info)).thenReturn(copiedProps);

        AwsCredentialsProvider credentialsProvider = mock(AwsCredentialsProvider.class);
        credentialsManagerMock
                .when(() -> AuroraDsqlCredentialsManager.getProvider())
                .thenReturn(credentialsProvider);

        // Mock ConnWrapper to prevent actual network connection
        Connection mockConnection = mock(Connection.class);
        try (MockedConstruction<ConnWrapper> connWrapperMock =
                mockConstruction(
                        ConnWrapper.class,
                        (mock, context) -> {
                            when(mock.makeConnection()).thenReturn(mockConnection);
                        })) {
            // Act
            Connection result = driver.connect(url, info);

            // Assert
            assertNotNull(result);
        }
    }

    @Test
    void testAcceptsURL_EdgeCases() throws SQLException {
        // Test edge cases and boundary conditions - only jdbc:aws-dsql:postgresql:// should return
        // true
        assertTrue(
                driver.acceptsURL(
                        "jdbc:aws-dsql:postgresql://a.dsql.us-east-1.on.aws/db")); // Single char
        // cluster

        // Test URLs that were previously supported but now should return false
        assertFalse(driver.acceptsURL("a.dsql.us-east-1.on.aws/db")); // Raw DSQL URL
        assertFalse(
                driver.acceptsURL(
                        "jdbc:postgresql://a.dsql.us-east-1.on.aws/db")); // Standard PostgreSQL

        // Test URLs that start with jdbc:postgresql but aren't DSQL
        assertFalse(driver.acceptsURL("jdbc:postgresql://regular.postgres.com/db"));

        // Test malformed URLs that might cause exceptions in isDsqlUrl
        assertFalse(driver.acceptsURL("jdbc:postgresql://[invalid"));
    }
}
