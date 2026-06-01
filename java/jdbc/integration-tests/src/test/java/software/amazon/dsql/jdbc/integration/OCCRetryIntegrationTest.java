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

import java.io.PrintWriter;
import java.sql.Connection;
import java.sql.DriverManager;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.sql.Statement;
import java.util.Properties;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicReference;
import java.util.logging.Logger;
import javax.sql.DataSource;
import org.junit.jupiter.api.AfterAll;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestInstance;
import software.amazon.dsql.jdbc.OCCRetry;
import software.amazon.dsql.jdbc.OCCRetryConfig;
import software.amazon.dsql.jdbc.OCCTransactionRunner;

/**
 * Integration tests for OCC retry against a live Aurora DSQL cluster.
 *
 * <p>Environment variables required:
 * <li>CLUSTER_ENDPOINT: Aurora DSQL cluster endpoint
 * <li>CLUSTER_USER: Database user (default: admin)
 */
@TestInstance(TestInstance.Lifecycle.PER_CLASS)
public class OCCRetryIntegrationTest {

    private static final String CLUSTER_ENDPOINT = System.getenv("CLUSTER_ENDPOINT");
    private static final String CLUSTER_USER = System.getenv("CLUSTER_USER");
    private final String tableName = "occ_retry_" + Long.toHexString(System.nanoTime());

    private static final OCCRetryConfig SETUP_CONFIG =
            OCCRetryConfig.builder().maxRetries(10).baseDelayMs(100).maxDelayMs(2000).build();

    @BeforeAll
    void setUp() throws SQLException {
        assertNotNull(CLUSTER_ENDPOINT, "CLUSTER_ENDPOINT environment variable must be set");

        try (Connection conn = createConnection()) {
            conn.setAutoCommit(true);
            try (Statement stmt = conn.createStatement()) {
                stmt.executeUpdate(
                        "CREATE TABLE IF NOT EXISTS "
                                + tableName
                                + " (id INT PRIMARY KEY, value INT)");
            }
        }
    }

    @AfterAll
    void tearDown() {
        try (Connection conn = createConnection()) {
            conn.setAutoCommit(true);
            try (Statement stmt = conn.createStatement()) {
                stmt.executeUpdate("DROP TABLE IF EXISTS " + tableName);
            }
        } catch (SQLException e) {
            // best-effort cleanup
        }
    }

    private Connection createConnection() throws SQLException {
        String user = CLUSTER_USER != null ? CLUSTER_USER : "admin";
        String url = "jdbc:aws-dsql:postgresql://" + CLUSTER_ENDPOINT + "/postgres";
        Properties props = new Properties();
        props.setProperty("user", user);
        return DriverManager.getConnection(url, props);
    }

    private DataSource createDataSource() {
        return new DataSource() {
            public Connection getConnection() throws SQLException {
                return createConnection();
            }

            public Connection getConnection(String username, String password) throws SQLException {
                return createConnection();
            }

            public PrintWriter getLogWriter() {
                return null;
            }

            public void setLogWriter(PrintWriter out) {}

            public void setLoginTimeout(int seconds) {}

            public int getLoginTimeout() {
                return 0;
            }

            public Logger getParentLogger() {
                return Logger.getLogger("software.amazon.dsql.jdbc");
            }

            public <T> T unwrap(Class<T> iface) throws SQLException {
                throw new SQLException("not a wrapper");
            }

            public boolean isWrapperFor(Class<?> iface) {
                return false;
            }
        };
    }

    private void resetRow(int id, int value) throws SQLException {
        OCCRetry.execute(
                createDataSource(),
                SETUP_CONFIG,
                conn -> {
                    try (Statement stmt = conn.createStatement()) {
                        stmt.executeUpdate(
                                "INSERT INTO "
                                        + tableName
                                        + " (id, value) VALUES ("
                                        + id
                                        + ", "
                                        + value
                                        + ")"
                                        + " ON CONFLICT (id) DO UPDATE SET value = "
                                        + value);
                    }
                    return null;
                });
    }

    @Test
    void testOCCRetryDeterministicConflict() throws Exception {
        OCCRetryConfig config =
                OCCRetryConfig.builder().maxRetries(10).baseDelayMs(50).maxDelayMs(1000).build();

        resetRow(1, 0);

        CountDownLatch t1HasRead = new CountDownLatch(1);
        CountDownLatch t2HasCommitted = new CountDownLatch(1);
        AtomicInteger t1Attempts = new AtomicInteger(0);
        AtomicReference<Exception> t2Error = new AtomicReference<>();

        Thread t2 =
                new Thread(
                        () -> {
                            try {
                                assertTrue(
                                        t1HasRead.await(30, TimeUnit.SECONDS),
                                        "T1 should have read");
                                OCCRetry.execute(
                                        createDataSource(),
                                        SETUP_CONFIG,
                                        conn -> {
                                            try (Statement stmt = conn.createStatement()) {
                                                stmt.executeUpdate(
                                                        "UPDATE "
                                                                + tableName
                                                                + " SET value = 100 WHERE id = 1");
                                            }
                                            return null;
                                        });
                            } catch (Exception e) {
                                t2Error.set(e);
                            } finally {
                                t2HasCommitted.countDown();
                            }
                        });
        t2.start();

        try (Connection conn = createConnection()) {
            Integer result =
                    OCCRetry.execute(
                            conn,
                            config,
                            c -> {
                                int attempt = t1Attempts.incrementAndGet();
                                int currentValue;
                                try (Statement stmt = c.createStatement();
                                        ResultSet rs =
                                                stmt.executeQuery(
                                                        "SELECT value FROM "
                                                                + tableName
                                                                + " WHERE id = 1")) {
                                    assertTrue(rs.next());
                                    currentValue = rs.getInt("value");
                                }

                                if (attempt == 1) {
                                    t1HasRead.countDown();
                                    try {
                                        assertTrue(
                                                t2HasCommitted.await(30, TimeUnit.SECONDS),
                                                "T2 should have committed");
                                    } catch (InterruptedException ie) {
                                        Thread.currentThread().interrupt();
                                        throw new SQLException("interrupted", ie);
                                    }
                                }

                                try (Statement stmt = c.createStatement()) {
                                    stmt.executeUpdate(
                                            "UPDATE "
                                                    + tableName
                                                    + " SET value = "
                                                    + (currentValue + 10)
                                                    + " WHERE id = 1");
                                }
                                return currentValue + 10;
                            });

            assertNotNull(result);
        }

        t2.join(30_000);
        assertFalse(t2.isAlive(), "T2 should have finished");
        if (t2Error.get() != null) {
            throw t2Error.get();
        }

        assertTrue(
                t1Attempts.get() >= 2,
                "Expected OCC retry but T1 ran only " + t1Attempts.get() + " time(s)");

        try (Connection conn = createConnection();
                Statement stmt = conn.createStatement();
                ResultSet rs =
                        stmt.executeQuery("SELECT value FROM " + tableName + " WHERE id = 1")) {
            assertTrue(rs.next());
            assertEquals(110, rs.getInt("value"));
        }
    }

    @Test
    void testOCCRetryConcurrentIncrements() throws Exception {
        OCCRetryConfig config =
                OCCRetryConfig.builder().maxRetries(10).baseDelayMs(50).maxDelayMs(1000).build();

        resetRow(2, 0);

        int numThreads = 3;
        AtomicInteger totalAttempts = new AtomicInteger(0);
        AtomicReference<Exception> firstError = new AtomicReference<>();
        Thread[] threads = new Thread[numThreads];

        for (int i = 0; i < numThreads; i++) {
            threads[i] =
                    new Thread(
                            () -> {
                                try (Connection conn = createConnection()) {
                                    OCCRetry.execute(
                                            conn,
                                            config,
                                            c -> {
                                                totalAttempts.incrementAndGet();
                                                int current;
                                                try (Statement stmt = c.createStatement();
                                                        ResultSet rs =
                                                                stmt.executeQuery(
                                                                        "SELECT value FROM "
                                                                                + tableName
                                                                                + " WHERE id = 2")) {
                                                    assertTrue(rs.next());
                                                    current = rs.getInt("value");
                                                }
                                                try (Statement stmt = c.createStatement()) {
                                                    stmt.executeUpdate(
                                                            "UPDATE "
                                                                    + tableName
                                                                    + " SET value = "
                                                                    + (current + 1)
                                                                    + " WHERE id = 2");
                                                }
                                                return null;
                                            });
                                } catch (Exception e) {
                                    firstError.compareAndSet(null, e);
                                }
                            });
            threads[i].start();
        }

        for (Thread t : threads) {
            t.join(60_000);
            assertFalse(t.isAlive(), "Thread should have finished");
        }

        if (firstError.get() != null) {
            throw firstError.get();
        }

        assertTrue(
                totalAttempts.get() > numThreads,
                "Expected OCC retries but total attempts ("
                        + totalAttempts.get()
                        + ") equals thread count — no contention occurred");

        try (Connection conn = createConnection();
                Statement stmt = conn.createStatement();
                ResultSet rs =
                        stmt.executeQuery("SELECT value FROM " + tableName + " WHERE id = 2")) {
            assertTrue(rs.next());
            assertEquals(numThreads, rs.getInt("value"));
        }
    }

    @Test
    void testOCCRetryWithDataSource() throws Exception {
        OCCRetryConfig config =
                OCCRetryConfig.builder().maxRetries(10).baseDelayMs(50).maxDelayMs(1000).build();
        DataSource ds = createDataSource();

        resetRow(3, 0);

        CountDownLatch t1HasRead = new CountDownLatch(1);
        CountDownLatch t2HasCommitted = new CountDownLatch(1);
        AtomicInteger t1Attempts = new AtomicInteger(0);
        AtomicReference<Exception> t2Error = new AtomicReference<>();

        Thread t2 =
                new Thread(
                        () -> {
                            try {
                                assertTrue(
                                        t1HasRead.await(30, TimeUnit.SECONDS),
                                        "T1 should have read");
                                OCCRetry.execute(
                                        createDataSource(),
                                        SETUP_CONFIG,
                                        conn -> {
                                            try (Statement stmt = conn.createStatement()) {
                                                stmt.executeUpdate(
                                                        "UPDATE "
                                                                + tableName
                                                                + " SET value = 50 WHERE id = 3");
                                            }
                                            return null;
                                        });
                            } catch (Exception e) {
                                t2Error.set(e);
                            } finally {
                                t2HasCommitted.countDown();
                            }
                        });
        t2.start();

        Integer result =
                OCCRetry.execute(
                        ds,
                        config,
                        c -> {
                            int attempt = t1Attempts.incrementAndGet();
                            int currentValue;
                            try (Statement stmt = c.createStatement();
                                    ResultSet rs =
                                            stmt.executeQuery(
                                                    "SELECT value FROM "
                                                            + tableName
                                                            + " WHERE id = 3")) {
                                assertTrue(rs.next());
                                currentValue = rs.getInt("value");
                            }

                            if (attempt == 1) {
                                t1HasRead.countDown();
                                try {
                                    assertTrue(
                                            t2HasCommitted.await(30, TimeUnit.SECONDS),
                                            "T2 should have committed");
                                } catch (InterruptedException ie) {
                                    Thread.currentThread().interrupt();
                                    throw new SQLException("interrupted", ie);
                                }
                            }

                            try (Statement stmt = c.createStatement()) {
                                stmt.executeUpdate(
                                        "UPDATE "
                                                + tableName
                                                + " SET value = "
                                                + (currentValue + 5)
                                                + " WHERE id = 3");
                            }
                            return currentValue + 5;
                        });

        t2.join(30_000);
        assertFalse(t2.isAlive(), "T2 should have finished");
        if (t2Error.get() != null) {
            throw t2Error.get();
        }

        assertTrue(t1Attempts.get() >= 2, "Expected OCC retry via DataSource");
        assertEquals(55, result);
    }

    @Test
    void testOCCTransactionRunnerVoid() throws SQLException {
        DataSource ds = createDataSource();
        OCCTransactionRunner runner = OCCTransactionRunner.create(ds, SETUP_CONFIG);

        resetRow(4, 0);

        runner.runVoid(
                c -> {
                    try (Statement stmt = c.createStatement()) {
                        stmt.executeUpdate("UPDATE " + tableName + " SET value = 42 WHERE id = 4");
                    }
                });

        try (Connection conn = createConnection();
                Statement stmt = conn.createStatement();
                ResultSet rs =
                        stmt.executeQuery("SELECT value FROM " + tableName + " WHERE id = 4")) {
            assertTrue(rs.next());
            assertEquals(42, rs.getInt("value"));
        }
    }

    @Test
    void testNonOCCErrorPropagatesImmediately() throws SQLException {
        OCCRetryConfig config =
                OCCRetryConfig.builder().maxRetries(5).baseDelayMs(10).maxDelayMs(200).build();
        AtomicInteger attempts = new AtomicInteger(0);

        try (Connection conn = createConnection()) {
            SQLException thrown =
                    assertThrows(
                            SQLException.class,
                            () ->
                                    OCCRetry.execute(
                                            conn,
                                            config,
                                            c -> {
                                                attempts.incrementAndGet();
                                                try (Statement stmt = c.createStatement()) {
                                                    stmt.executeQuery(
                                                            "SELECT * FROM nonexistent_table_xyz");
                                                }
                                                return null;
                                            }));

            assertEquals(1, attempts.get(), "Non-OCC error should not trigger retry");
            assertFalse(OCCRetry.isOCCError(thrown), "Error should not be classified as OCC");
        }
    }
}
