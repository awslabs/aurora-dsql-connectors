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

import java.sql.Connection;
import java.sql.SQLException;
import java.util.Objects;
import java.util.concurrent.ThreadLocalRandom;
import javax.sql.DataSource;

/**
 * Executes a transaction callback with automatic retry on OCC (Optimistic Concurrency Control)
 * conflicts.
 *
 * <p>Aurora DSQL uses OCC where conflicts are detected at commit time. When two transactions modify
 * the same data concurrently, the second to commit receives an OCC error. This utility
 * automatically retries the transaction with exponential backoff.
 *
 * <h3>Usage with connection pool (recommended)</h3>
 *
 * <pre>{@code
 * DataSource ds = new HikariDataSource(config);
 * OCCRetryConfig retryConfig = OCCRetryConfig.defaults();
 *
 * int result = OCCRetry.execute(ds, retryConfig, conn -> {
 *     Statement stmt = conn.createStatement();
 *     stmt.executeUpdate("UPDATE accounts SET balance = balance - 100 WHERE id = 1");
 *     stmt.executeUpdate("UPDATE accounts SET balance = balance + 100 WHERE id = 2");
 *     return 2;
 * });
 * }</pre>
 *
 * <h3>Usage with direct connection</h3>
 *
 * <pre>{@code
 * Connection conn = DriverManager.getConnection(url, props);
 * OCCRetry.execute(conn, retryConfig, c -> {
 *     c.createStatement().executeUpdate("UPDATE accounts SET ...");
 *     return null;
 * });
 * }</pre>
 *
 * <p><b>Idempotency warning:</b> The callback may be called multiple times on OCC conflicts. Ensure
 * it has no side effects that should not be repeated (e.g., sending emails, incrementing external
 * counters).
 */
public final class OCCRetry {

    private static final String OCC_CODE_MUTATION = "OC000";
    private static final String OCC_CODE_SCHEMA = "OC001";
    private static final String OCC_CODE_SERIALIZATION = "40001";

    private OCCRetry() {
        throw new UnsupportedOperationException(
                "This is a utility class and cannot be instantiated");
    }

    /**
     * Callback interface for transaction work to be executed with OCC retry.
     *
     * @param <T> the return type of the transaction
     */
    @FunctionalInterface
    public interface TransactionCallback<T> {
        /**
         * Executes transaction work on the provided connection.
         *
         * <p>The connection has auto-commit disabled. Do not call {@code commit()} or {@code
         * rollback()} — the retry utility manages the transaction boundaries.
         *
         * @param connection the connection to execute work on
         * @return the result of the transaction
         * @throws SQLException if a database error occurs
         */
        T execute(Connection connection) throws SQLException;
    }

    /**
     * Executes a transaction with OCC retry, acquiring a fresh connection from the DataSource on
     * each attempt.
     *
     * @param <T> the return type of the transaction
     * @param dataSource the DataSource to acquire connections from
     * @param config retry configuration
     * @param callback the transaction work to execute
     * @return the result of the successful transaction
     * @throws SQLException if a non-OCC error occurs or retries are exhausted
     */
    public static <T> T execute(
            DataSource dataSource, OCCRetryConfig config, TransactionCallback<T> callback)
            throws SQLException {
        Objects.requireNonNull(dataSource, "dataSource must not be null");
        Objects.requireNonNull(config, "config must not be null");
        Objects.requireNonNull(callback, "callback must not be null");

        SQLException lastError = null;
        for (int attempt = 0; attempt <= config.getMaxRetries(); attempt++) {
            try (Connection conn = dataSource.getConnection()) {
                conn.setAutoCommit(false);
                T result = callback.execute(conn);
                conn.commit();
                return result;
            } catch (SQLException e) {
                if (!isOCCError(e)) {
                    throw e;
                }
                lastError = e;
                if (attempt < config.getMaxRetries()) {
                    sleep(calculateBackoff(config, attempt));
                }
            }
        }
        throw lastError;
    }

    /**
     * Executes a transaction with OCC retry on an existing connection.
     *
     * <p>The connection is reused across retry attempts (rolled back between attempts). The caller
     * retains ownership — the connection is not closed by this method.
     *
     * @param <T> the return type of the transaction
     * @param connection the connection to execute on
     * @param config retry configuration
     * @param callback the transaction work to execute
     * @return the result of the successful transaction
     * @throws SQLException if a non-OCC error occurs or retries are exhausted
     */
    public static <T> T execute(
            Connection connection, OCCRetryConfig config, TransactionCallback<T> callback)
            throws SQLException {
        Objects.requireNonNull(connection, "connection must not be null");
        Objects.requireNonNull(config, "config must not be null");
        Objects.requireNonNull(callback, "callback must not be null");

        connection.setAutoCommit(false);
        SQLException lastError = null;
        for (int attempt = 0; attempt <= config.getMaxRetries(); attempt++) {
            try {
                T result = callback.execute(connection);
                connection.commit();
                return result;
            } catch (SQLException e) {
                try {
                    connection.rollback();
                } catch (SQLException rollbackEx) {
                    e.addSuppressed(rollbackEx);
                }
                if (!isOCCError(e)) {
                    throw e;
                }
                lastError = e;
                if (attempt < config.getMaxRetries()) {
                    sleep(calculateBackoff(config, attempt));
                }
            }
        }
        throw lastError;
    }

    static long calculateBackoff(OCCRetryConfig config, int attempt) {
        int exponent = Math.min(attempt, 31);
        double delay =
                Math.min(
                        config.getBaseDelayMs() * Math.pow(config.getMultiplier(), exponent),
                        config.getMaxDelayMs());
        double jitter = delay * ThreadLocalRandom.current().nextDouble() * config.getJitterFactor();
        return (long) (delay + jitter);
    }

    private static void sleep(long millis) throws SQLException {
        try {
            Thread.sleep(millis);
        } catch (InterruptedException ie) {
            Thread.currentThread().interrupt();
            throw new SQLException("Retry interrupted", ie);
        }
    }

    /**
     * Checks if a SQLException is an OCC conflict error.
     *
     * <p>Returns true for SQLState OC000 (mutation conflict), OC001 (schema conflict), or 40001
     * (serialization failure). SQLState 40001 is matched unconditionally because Aurora DSQL
     * surfaces OCC conflicts with this code. Also inspects the exception message for OC000/OC001
     * codes, as pgJDBC may surface these in the message text with 40001 as the envelope SQLState.
     *
     * @param e the exception to check
     * @return true if the error is an OCC conflict
     */
    public static boolean isOCCError(SQLException e) {
        if (e == null) {
            return false;
        }
        String sqlState = e.getSQLState();
        if (OCC_CODE_MUTATION.equals(sqlState) || OCC_CODE_SCHEMA.equals(sqlState)) {
            return true;
        }
        String message = e.getMessage();
        if (message != null
                && (message.contains(OCC_CODE_MUTATION) || message.contains(OCC_CODE_SCHEMA))) {
            return true;
        }
        return OCC_CODE_SERIALIZATION.equals(sqlState);
    }
}
