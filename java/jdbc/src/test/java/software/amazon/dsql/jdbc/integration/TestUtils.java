package software.amazon.dsql.jdbc.integration;

import java.sql.Connection;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.sql.Statement;
import java.util.Properties;

/**
 * Utility class for Aurora DSQL JDBC Connector integration tests.
 */
public class TestUtils {
    
    private static final String CLUSTER_ENDPOINT = System.getenv("CLUSTER_ENDPOINT");
    private static final String REGION = System.getenv("REGION");
    private static final String CLUSTER_USER = System.getenv("CLUSTER_USER");
    
    /**
     * Check if integration tests are enabled and required environment variables are set.
     */
    public static boolean isIntegrationTestEnabled() {
        String runIntegration = System.getenv("RUN_INTEGRATION");
        return "TRUE".equalsIgnoreCase(runIntegration) && 
               CLUSTER_ENDPOINT != null && 
               REGION != null;
    }
    
    /**
     * Get the cluster endpoint from environment variables.
     */
    public static String getClusterEndpoint() {
        return CLUSTER_ENDPOINT;
    }
    
    /**
     * Get the region from environment variables.
     */
    public static String getRegion() {
        return REGION;
    }
    
    /**
     * Get the cluster user from environment variables, defaulting to "admin".
     */
    public static String getClusterUser() {
        return CLUSTER_USER != null ? CLUSTER_USER : "admin";
    }
    
    /**
     * Create a basic connection URL for Aurora DSQL.
     */
    public static String createConnectionUrl() {
        return createConnectionUrl(getClusterUser());
    }
    
    /**
     * Create a connection URL for Aurora DSQL with specified user.
     */
    public static String createConnectionUrl(String user) {
        return "jdbc:postgresql://" + CLUSTER_ENDPOINT + "/postgres?user=" + user;
    }
    
    /**
     * Create connection properties for Aurora DSQL.
     */
    public static Properties createConnectionProperties() {
        return createConnectionProperties(getClusterUser());
    }
    
    /**
     * Create connection properties for Aurora DSQL with specified user.
     */
    public static Properties createConnectionProperties(String user) {
        Properties props = new Properties();
        props.setProperty("user", user);
        return props;
    }
    
    /**
     * Create connection properties with custom token duration.
     */
    public static Properties createConnectionProperties(String user, int tokenDurationSecs) {
        Properties props = createConnectionProperties(user);
        props.setProperty("token-duration-secs", String.valueOf(tokenDurationSecs));
        return props;
    }
    
    /**
     * Execute a simple test query to verify connection is working.
     */
    public static void verifyConnection(Connection conn) throws SQLException {
        try (Statement stmt = conn.createStatement();
             ResultSet rs = stmt.executeQuery("SELECT 1 as test_value")) {
            if (!rs.next()) {
                throw new SQLException("Test query returned no results");
            }
            int testValue = rs.getInt("test_value");
            if (testValue != 1) {
                throw new SQLException("Test query returned unexpected value: " + testValue);
            }
        }
    }
    
    /**
     * Get database information for logging/debugging.
     */
    public static String getDatabaseInfo(Connection conn) throws SQLException {
        try (Statement stmt = conn.createStatement();
             ResultSet rs = stmt.executeQuery("SELECT current_user, current_database(), version()")) {
            if (rs.next()) {
                return String.format("User: %s, Database: %s, Version: %s", 
                    rs.getString(1), rs.getString(2), rs.getString(3));
            }
            return "Unable to retrieve database info";
        }
    }
    
    /**
     * Create a temporary table for testing.
     */
    public static void createTestTable(Connection conn, String tableName) throws SQLException {
        try (Statement stmt = conn.createStatement()) {
            stmt.executeUpdate(String.format(
                "CREATE TEMPORARY TABLE %s (id INTEGER PRIMARY KEY, name VARCHAR(100), created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)",
                tableName));
        }
    }
    
    /**
     * Insert test data into a table.
     */
    public static void insertTestData(Connection conn, String tableName, int id, String name) throws SQLException {
        try (Statement stmt = conn.createStatement()) {
            stmt.executeUpdate(String.format(
                "INSERT INTO %s (id, name) VALUES (%d, '%s')",
                tableName, id, name));
        }
    }
    
    /**
     * Count rows in a table.
     */
    public static int countRows(Connection conn, String tableName) throws SQLException {
        try (Statement stmt = conn.createStatement();
             ResultSet rs = stmt.executeQuery("SELECT COUNT(*) FROM " + tableName)) {
            if (rs.next()) {
                return rs.getInt(1);
            }
            return 0;
        }
    }
    
    /**
     * Clean up test data by dropping a temporary table.
     */
    public static void dropTestTable(Connection conn, String tableName) throws SQLException {
        try (Statement stmt = conn.createStatement()) {
            stmt.executeUpdate("DROP TABLE IF EXISTS " + tableName);
        }
    }
    
    /**
     * Print test environment information.
     */
    public static void printTestEnvironment() {
        System.out.println("=== Aurora DSQL JDBC Connector Integration Test Environment ===");
        System.out.println("Cluster Endpoint: " + (CLUSTER_ENDPOINT != null ? CLUSTER_ENDPOINT : "NOT SET"));
        System.out.println("Region: " + (REGION != null ? REGION : "NOT SET"));
        System.out.println("Cluster User: " + getClusterUser());
        System.out.println("Integration Tests Enabled: " + isIntegrationTestEnabled());
        System.out.println("Java Version: " + System.getProperty("java.version"));
        System.out.println("OS: " + System.getProperty("os.name") + " " + System.getProperty("os.version"));
        System.out.println("===============================================================");
    }
    
    /**
     * Wait for a specified duration (for testing timing-related functionality).
     */
    public static void waitSeconds(int seconds) {
        try {
            Thread.sleep(seconds * 1000L);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            throw new RuntimeException("Interrupted while waiting", e);
        }
    }
}
