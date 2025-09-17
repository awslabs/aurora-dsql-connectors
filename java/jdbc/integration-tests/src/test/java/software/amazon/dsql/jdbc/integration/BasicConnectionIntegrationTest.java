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

package software.amazon.dsql.jdbc.integration;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assertions.fail;

import java.sql.Connection;
import java.sql.DatabaseMetaData;
import java.sql.DriverManager;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.sql.Statement;
import java.util.Properties;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestInstance;

/**
 * Integration tests for basic Aurora DSQL JDBC Connector connections. These tests require a live
 * Aurora DSQL cluster and AWS credentials.
 *
 * <p>Environment variables required:
 * <li>CLUSTER_ENDPOINT: Aurora DSQL cluster endpoint
 * <li>REGION: AWS region (e.g., us-east-1)
 * <li>CLUSTER_USER: Database user (default: admin)
 */
@TestInstance(TestInstance.Lifecycle.PER_CLASS)
public class BasicConnectionIntegrationTest {

    private static final String CLUSTER_ENDPOINT = System.getenv("CLUSTER_ENDPOINT");
    private static final String REGION = System.getenv("REGION");
    private static final String CLUSTER_USER = System.getenv("CLUSTER_USER");

    @BeforeAll
    void setUp() {
        // Validate required environment variables
        assertNotNull(CLUSTER_ENDPOINT, "CLUSTER_ENDPOINT environment variable must be set");
        assertNotNull(REGION, "REGION environment variable must be set");

        System.out.println("Running integration tests against cluster: " + CLUSTER_ENDPOINT);
        System.out.println("Region: " + REGION);
        System.out.println("User: " + (CLUSTER_USER != null ? CLUSTER_USER : "admin"));
    }

    @Test
    void testBasicConnectionWithEndpoint() throws SQLException {
        String user = CLUSTER_USER != null ? CLUSTER_USER : "admin";
        String url = "jdbc:aws-dsql:postgresql://" + CLUSTER_ENDPOINT + "/postgres?user=" + user;

        try (Connection conn = DriverManager.getConnection(url)) {
            assertNotNull(conn, "Connection should not be null");
            assertFalse(conn.isClosed(), "Connection should be open");

            // Test basic query
            try (Statement stmt = conn.createStatement();
                    ResultSet rs = stmt.executeQuery("SELECT 1 as test_value")) {
                assertTrue(rs.next(), "Result set should have at least one row");
                assertEquals(1, rs.getInt("test_value"), "Test value should be 1");
            }
        }
    }

    @Test
    void testConnectionWithProperties() throws SQLException {
        String user = CLUSTER_USER != null ? CLUSTER_USER : "admin";
        Properties props = new Properties();
        props.setProperty("user", user);

        String url = "jdbc:aws-dsql:postgresql://" + CLUSTER_ENDPOINT + "/postgres";

        try (Connection conn = DriverManager.getConnection(url, props)) {
            assertNotNull(conn, "Connection should not be null");
            assertFalse(conn.isClosed(), "Connection should be open");

            // Test database metadata
            String databaseProductName = conn.getMetaData().getDatabaseProductName();
            assertTrue(
                    databaseProductName.contains("PostgreSQL"),
                    "Database product should be PostgreSQL-compatible");
        }
    }

    @Test
    void testConnectionWithCustomTokenDuration() throws SQLException {
        String user = CLUSTER_USER != null ? CLUSTER_USER : "admin";
        String url =
                "jdbc:aws-dsql:postgresql://"
                        + CLUSTER_ENDPOINT
                        + "/postgres?user="
                        + user
                        + "&token-duration-secs=14400";

        try (Connection conn = DriverManager.getConnection(url)) {
            assertNotNull(conn, "Connection should not be null");
            assertFalse(conn.isClosed(), "Connection should be open");

            // Test that connection works with custom token duration
            try (Statement stmt = conn.createStatement();
                    ResultSet rs = stmt.executeQuery("SELECT current_timestamp")) {
                assertTrue(rs.next(), "Result set should have at least one row");
                assertNotNull(rs.getTimestamp(1), "Timestamp should not be null");
            }
        }
    }

    @Test
    void testMultipleConnections() throws SQLException {
        String user = CLUSTER_USER != null ? CLUSTER_USER : "admin";
        String url = "jdbc:aws-dsql:postgresql://" + CLUSTER_ENDPOINT + "/postgres?user=" + user;

        // Test multiple concurrent connections
        try (Connection conn1 = DriverManager.getConnection(url);
                Connection conn2 = DriverManager.getConnection(url);
                Connection conn3 = DriverManager.getConnection(url)) {

            assertNotNull(conn1, "Connection 1 should not be null");
            assertNotNull(conn2, "Connection 2 should not be null");
            assertNotNull(conn3, "Connection 3 should not be null");

            assertFalse(conn1.isClosed(), "Connection 1 should be open");
            assertFalse(conn2.isClosed(), "Connection 2 should be open");
            assertFalse(conn3.isClosed(), "Connection 3 should be open");

            // Test that all connections work independently
            for (int i = 1; i <= 3; i++) {
                Connection conn = (i == 1) ? conn1 : (i == 2) ? conn2 : conn3;
                try (Statement stmt = conn.createStatement();
                        ResultSet rs = stmt.executeQuery("SELECT " + i + " as connection_id")) {
                    assertTrue(
                            rs.next(),
                            "Result set should have at least one row for connection " + i);
                    assertEquals(
                            i,
                            rs.getInt("connection_id"),
                            "Connection ID should match for connection " + i);
                }
            }
        }
    }

    @Test
    void testTransactionHandling() throws SQLException {
        String user = CLUSTER_USER != null ? CLUSTER_USER : "admin";
        String url = "jdbc:aws-dsql:postgresql://" + CLUSTER_ENDPOINT + "/postgres?user=" + user;

        try (Connection conn = DriverManager.getConnection(url)) {
            assertNotNull(conn, "Connection should not be null");

            // Test transaction with auto-commit disabled
            conn.setAutoCommit(false);
            assertFalse(conn.getAutoCommit(), "Auto-commit should be disabled");

            try (Statement stmt = conn.createStatement()) {
                // Create a temporary table for testing
                stmt.executeUpdate("DROP TABLE IF EXISTS test_table");
                // Commit the transaction
                conn.commit();
            }

            try (Statement stmt = conn.createStatement()) {
                // Create a temporary table for testing
                stmt.executeUpdate(
                        "CREATE TABLE IF NOT EXISTS test_table (id INTEGER, name VARCHAR(50))");
                // Commit the transaction
                conn.commit();
            }

            try (Statement stmt = conn.createStatement()) {
                // Insert test data
                stmt.executeUpdate("INSERT INTO test_table (id, name) VALUES (1, 'test1')");
                stmt.executeUpdate("INSERT INTO test_table (id, name) VALUES (2, 'test2')");

                // Commit the transaction
                conn.commit();

                // Verify data was committed
                try (ResultSet rs = stmt.executeQuery("SELECT COUNT(*) FROM test_table")) {
                    assertTrue(rs.next(), "Result set should have at least one row");
                    assertEquals(2, rs.getInt(1), "Should have 2 rows in test table");
                }
            }

            // Reset auto-commit
            conn.setAutoCommit(true);
            assertTrue(conn.getAutoCommit(), "Auto-commit should be enabled");
        }
    }

    @Test
    void testPreparedStatements() throws SQLException {
        String user = CLUSTER_USER != null ? CLUSTER_USER : "admin";
        String url = "jdbc:aws-dsql:postgresql://" + CLUSTER_ENDPOINT + "/postgres?user=" + user;

        try (Connection conn = DriverManager.getConnection(url)) {
            assertNotNull(conn, "Connection should not be null");

            // Create a temporary table
            try (Statement stmt = conn.createStatement()) {
                stmt.executeUpdate(
                        "CREATE TABLE IF NOT EXISTS prep_test (id INTEGER, value VARCHAR(100))");
            }

            // Test prepared statement insertion
            String insertSql = "INSERT INTO prep_test (id, value) VALUES (?, ?)";
            try (PreparedStatement pstmt = conn.prepareStatement(insertSql)) {
                for (int i = 1; i <= 5; i++) {
                    pstmt.setInt(1, i);
                    pstmt.setString(2, "value_" + i);
                    pstmt.executeUpdate();
                }
            }

            // Test prepared statement query
            String selectSql = "SELECT * FROM prep_test WHERE id = ?";
            try (PreparedStatement pstmt = conn.prepareStatement(selectSql)) {
                pstmt.setInt(1, 3);
                try (ResultSet rs = pstmt.executeQuery()) {
                    assertTrue(rs.next(), "Result set should have at least one row");
                    assertEquals(3, rs.getInt("id"), "ID should be 3");
                    assertEquals("value_3", rs.getString("value"), "Value should be 'value_3'");
                }
            }
        }
    }

    @Test
    void testConnectionMetadata() throws SQLException {
        String user = CLUSTER_USER != null ? CLUSTER_USER : "admin";
        String url = "jdbc:aws-dsql:postgresql://" + CLUSTER_ENDPOINT + "/postgres?user=" + user;

        try (Connection conn = DriverManager.getConnection(url)) {
            assertNotNull(conn, "Connection should not be null");

            DatabaseMetaData metadata = conn.getMetaData();
            assertNotNull(metadata, "Metadata should not be null");

            // Test basic metadata
            String databaseProductName = metadata.getDatabaseProductName();
            String databaseProductVersion = metadata.getDatabaseProductVersion();
            String driverName = metadata.getDriverName();
            String driverVersion = metadata.getDriverVersion();

            assertNotNull(databaseProductName, "Database product name should not be null");
            assertNotNull(databaseProductVersion, "Database product version should not be null");
            assertNotNull(driverName, "Driver name should not be null");
            assertNotNull(driverVersion, "Driver version should not be null");

            System.out.println("Database: " + databaseProductName + " " + databaseProductVersion);
            System.out.println("Driver: " + driverName + " " + driverVersion);

            // Test that we can get table information
            try (ResultSet rs = metadata.getTables(null, null, "%", new String[] {"TABLE"})) {
                // Should be able to execute without error
                assertNotNull(rs, "Table metadata result set should not be null");
            }
        }
    }

    @Test
    void testErrorHandling() {
        String user = CLUSTER_USER != null ? CLUSTER_USER : "admin";

        // Test with invalid endpoint
        String invalidUrl =
                "jdbc:aws-dsql:postgresql://invalid-endpoint.dsql.us-east-1.on.aws/postgres?user="
                        + user;

        assertThrows(
                SQLException.class,
                () -> {
                    try (Connection conn = DriverManager.getConnection(invalidUrl)) {
                        // Should not reach here
                        fail("Connection should fail with invalid endpoint");
                    }
                },
                "Should throw SQLException for invalid endpoint");
    }
}
