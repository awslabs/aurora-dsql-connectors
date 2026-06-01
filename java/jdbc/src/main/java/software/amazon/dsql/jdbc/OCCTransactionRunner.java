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

import java.sql.SQLException;
import java.util.Objects;
import javax.sql.DataSource;

/**
 * A reusable transaction runner that binds a DataSource and retry configuration once, then executes
 * transaction callbacks with OCC retry on each call.
 *
 * <p>Intentionally thin convenience over {@link OCCRetry#execute(DataSource, OCCRetryConfig,
 * OCCRetry.TransactionCallback)}.
 *
 * <pre>{@code
 * OCCTransactionRunner runner = OCCTransactionRunner.create(dataSource);
 *
 * int count = runner.run(conn -> {
 *     Statement stmt = conn.createStatement();
 *     return stmt.executeUpdate("UPDATE accounts SET balance = balance - 100 WHERE id = 1");
 * });
 *
 * runner.runVoid(conn -> {
 *     conn.createStatement().executeUpdate("DELETE FROM expired_sessions");
 * });
 * }</pre>
 */
public final class OCCTransactionRunner {

    private final DataSource dataSource;
    private final OCCRetryConfig config;

    private OCCTransactionRunner(DataSource dataSource, OCCRetryConfig config) {
        this.dataSource = Objects.requireNonNull(dataSource, "dataSource must not be null");
        this.config = Objects.requireNonNull(config, "config must not be null");
    }

    /**
     * Creates a runner with default retry configuration.
     *
     * @param dataSource the DataSource to acquire connections from
     * @return a new runner instance
     */
    public static OCCTransactionRunner create(DataSource dataSource) {
        return new OCCTransactionRunner(dataSource, OCCRetryConfig.defaults());
    }

    /**
     * Creates a runner with custom retry configuration.
     *
     * @param dataSource the DataSource to acquire connections from
     * @param config retry configuration
     * @return a new runner instance
     */
    public static OCCTransactionRunner create(DataSource dataSource, OCCRetryConfig config) {
        return new OCCTransactionRunner(dataSource, config);
    }

    /**
     * Executes a transaction callback with OCC retry.
     *
     * @param <T> the return type of the transaction
     * @param callback the transaction work to execute
     * @return the result of the successful transaction
     * @throws SQLException if a non-OCC error occurs or retries are exhausted
     */
    public <T> T run(OCCRetry.TransactionCallback<T> callback) throws SQLException {
        return OCCRetry.execute(dataSource, config, callback);
    }

    /**
     * Executes a void transaction callback with OCC retry.
     *
     * @param callback the transaction work to execute
     * @throws SQLException if a non-OCC error occurs or retries are exhausted
     */
    public void runVoid(VoidTransactionCallback callback) throws SQLException {
        OCCRetry.execute(
                dataSource,
                config,
                conn -> {
                    callback.execute(conn);
                    return null;
                });
    }
}
