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
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

import java.sql.Connection;
import java.sql.SQLException;
import java.util.concurrent.atomic.AtomicInteger;
import javax.sql.DataSource;
import org.junit.jupiter.api.Test;

class OCCRetryTest {

    @Test
    void isOCCError_null_returnsFalse() {
        assertFalse(OCCRetry.isOCCError(null));
    }

    @Test
    void isOCCError_nullSqlState_returnsFalse() {
        SQLException e = new SQLException("some error", (String) null);
        assertFalse(OCCRetry.isOCCError(e));
    }

    @Test
    void isOCCError_mutationConflict_returnsTrue() {
        SQLException e = new SQLException("conflict", "OC000");
        assertTrue(OCCRetry.isOCCError(e));
    }

    @Test
    void isOCCError_schemaConflict_returnsTrue() {
        SQLException e = new SQLException("conflict", "OC001");
        assertTrue(OCCRetry.isOCCError(e));
    }

    @Test
    void isOCCError_serializationFailure_returnsTrue() {
        SQLException e = new SQLException("conflict", "40001");
        assertTrue(OCCRetry.isOCCError(e));
    }

    @Test
    void isOCCError_unrecognizedState_returnsFalse() {
        SQLException e = new SQLException("something else", "42000");
        assertFalse(OCCRetry.isOCCError(e));
    }

    @Test
    void isOCCError_messageContainsOC000_returnsTrue() {
        SQLException e = new SQLException("ERROR: OC000 mutation conflict detected", "XX000");
        assertTrue(OCCRetry.isOCCError(e));
    }

    @Test
    void isOCCError_messageContainsOC001_returnsTrue() {
        SQLException e = new SQLException("ERROR: OC001 schema conflict", "XX000");
        assertTrue(OCCRetry.isOCCError(e));
    }

    @Test
    void isOCCError_serialization_withOC000InMessage_returnsTrue() {
        SQLException e = new SQLException("ERROR: OC000 mutation conflict detected", "40001");
        assertTrue(OCCRetry.isOCCError(e));
    }

    @Test
    void isOCCError_serialization_withOC001InMessage_returnsTrue() {
        SQLException e = new SQLException("ERROR: OC001 schema conflict", "40001");
        assertTrue(OCCRetry.isOCCError(e));
    }

    @Test
    void isOCCError_serialization_withoutOCCInMessage_returnsTrue() {
        SQLException e = new SQLException("serialization failure", "40001");
        assertTrue(OCCRetry.isOCCError(e));
    }

    @Test
    void isOCCError_nullMessage_returnsFalse() {
        SQLException e = new SQLException(null, "XX000");
        assertFalse(OCCRetry.isOCCError(e));
    }

    // --- execute(DataSource, config, callback) tests ---

    @Test
    void executeDataSource_successOnFirstAttempt() throws SQLException {
        DataSource ds = mock(DataSource.class);
        Connection conn = mock(Connection.class);
        when(ds.getConnection()).thenReturn(conn);

        String result = OCCRetry.execute(ds, OCCRetryConfig.defaults(), c -> "hello");

        assertEquals("hello", result);
        verify(conn).setAutoCommit(false);
        verify(conn).commit();
        verify(conn).close();
    }

    @Test
    void executeDataSource_retriesOnOCCThenSucceeds() throws SQLException {
        DataSource ds = mock(DataSource.class);
        Connection conn1 = mock(Connection.class);
        Connection conn2 = mock(Connection.class);
        when(ds.getConnection()).thenReturn(conn1, conn2);

        AtomicInteger attempts = new AtomicInteger();
        OCCRetryConfig config =
                OCCRetryConfig.builder().maxRetries(3).baseDelayMs(1).maxDelayMs(10).build();

        String result =
                OCCRetry.execute(
                        ds,
                        config,
                        c -> {
                            if (attempts.getAndIncrement() == 0) {
                                throw new SQLException("conflict", "OC000");
                            }
                            return "recovered";
                        });

        assertEquals("recovered", result);
        assertEquals(2, attempts.get());
        verify(conn1).close();
        verify(conn2).close();
    }

    @Test
    void executeDataSource_throwsNonOCCImmediately() throws SQLException {
        DataSource ds = mock(DataSource.class);
        Connection conn = mock(Connection.class);
        when(ds.getConnection()).thenReturn(conn);

        SQLException nonOcc = new SQLException("syntax error", "42601");
        OCCRetryConfig config = OCCRetryConfig.builder().baseDelayMs(1).maxDelayMs(10).build();

        SQLException thrown =
                assertThrows(
                        SQLException.class,
                        () ->
                                OCCRetry.execute(
                                        ds,
                                        config,
                                        c -> {
                                            throw nonOcc;
                                        }));

        assertEquals(nonOcc, thrown);
        verify(conn).close();
    }

    @Test
    void executeDataSource_exhaustsRetries() throws SQLException {
        DataSource ds = mock(DataSource.class);
        Connection conn = mock(Connection.class);
        when(ds.getConnection()).thenReturn(conn);

        AtomicInteger attempts = new AtomicInteger();
        OCCRetryConfig config =
                OCCRetryConfig.builder().maxRetries(2).baseDelayMs(1).maxDelayMs(10).build();

        SQLException thrown =
                assertThrows(
                        SQLException.class,
                        () ->
                                OCCRetry.execute(
                                        ds,
                                        config,
                                        c -> {
                                            attempts.incrementAndGet();
                                            throw new SQLException("conflict", "OC000");
                                        }));

        assertEquals(3, attempts.get()); // initial + 2 retries
        assertEquals("OC000", thrown.getSQLState());
    }

    // --- execute(Connection, config, callback) tests ---

    @Test
    void executeConnection_successOnFirstAttempt() throws SQLException {
        Connection conn = mock(Connection.class);

        int result = OCCRetry.execute(conn, OCCRetryConfig.defaults(), c -> 42);

        assertEquals(42, result);
        verify(conn).setAutoCommit(false);
        verify(conn).commit();
        verify(conn, never()).rollback();
        verify(conn, never()).close();
    }

    @Test
    void executeConnection_retriesOnOCCThenSucceeds() throws SQLException {
        Connection conn = mock(Connection.class);
        AtomicInteger attempts = new AtomicInteger();
        OCCRetryConfig config =
                OCCRetryConfig.builder().maxRetries(3).baseDelayMs(1).maxDelayMs(10).build();

        String result =
                OCCRetry.execute(
                        conn,
                        config,
                        c -> {
                            if (attempts.getAndIncrement() < 2) {
                                throw new SQLException("conflict", "OC000");
                            }
                            return "ok";
                        });

        assertEquals("ok", result);
        assertEquals(3, attempts.get());
        verify(conn, times(2)).rollback();
        verify(conn).commit();
        verify(conn, never()).close();
    }

    @Test
    void executeConnection_throwsNonOCCImmediately() throws SQLException {
        Connection conn = mock(Connection.class);
        SQLException nonOcc = new SQLException("constraint", "23505");
        OCCRetryConfig config = OCCRetryConfig.builder().baseDelayMs(1).maxDelayMs(10).build();

        SQLException thrown =
                assertThrows(
                        SQLException.class,
                        () ->
                                OCCRetry.execute(
                                        conn,
                                        config,
                                        c -> {
                                            throw nonOcc;
                                        }));

        assertEquals(nonOcc, thrown);
        verify(conn).rollback();
        verify(conn, never()).commit();
    }

    @Test
    void executeConnection_exhaustsRetries() throws SQLException {
        Connection conn = mock(Connection.class);
        AtomicInteger attempts = new AtomicInteger();
        OCCRetryConfig config =
                OCCRetryConfig.builder().maxRetries(1).baseDelayMs(1).maxDelayMs(10).build();

        SQLException thrown =
                assertThrows(
                        SQLException.class,
                        () ->
                                OCCRetry.execute(
                                        conn,
                                        config,
                                        c -> {
                                            attempts.incrementAndGet();
                                            throw new SQLException("conflict", "OC001");
                                        }));

        assertEquals(2, attempts.get()); // initial + 1 retry
        assertEquals("OC001", thrown.getSQLState());
        verify(conn, times(2)).rollback();
    }

    // --- commit-throwing-OCC tests ---

    @Test
    void executeDataSource_commitThrowsOCC_retries() throws SQLException {
        DataSource ds = mock(DataSource.class);
        Connection conn1 = mock(Connection.class);
        Connection conn2 = mock(Connection.class);
        when(ds.getConnection()).thenReturn(conn1, conn2);
        org.mockito.Mockito.doThrow(new SQLException("conflict", "OC000")).when(conn1).commit();

        OCCRetryConfig config =
                OCCRetryConfig.builder().maxRetries(3).baseDelayMs(1).maxDelayMs(10).build();

        String result = OCCRetry.execute(ds, config, c -> "done");

        assertEquals("done", result);
        verify(conn1).commit();
        verify(conn2).commit();
    }

    @Test
    void executeConnection_commitThrowsOCC_retries() throws SQLException {
        Connection conn = mock(Connection.class);
        AtomicInteger attempts = new AtomicInteger();
        org.mockito.Mockito.doAnswer(
                        invocation -> {
                            if (attempts.get() == 1) {
                                throw new SQLException("conflict", "40001");
                            }
                            return null;
                        })
                .when(conn)
                .commit();

        OCCRetryConfig config =
                OCCRetryConfig.builder().maxRetries(3).baseDelayMs(1).maxDelayMs(10).build();

        String result =
                OCCRetry.execute(
                        conn,
                        config,
                        c -> {
                            attempts.incrementAndGet();
                            return "ok";
                        });

        assertEquals("ok", result);
        assertEquals(2, attempts.get());
        verify(conn).rollback();
        verify(conn, times(2)).commit();
    }

    // --- InterruptedException test ---

    @Test
    void executeConnection_interruptedDuringSleep_throwsSQLException() throws SQLException {
        Connection conn = mock(Connection.class);
        OCCRetryConfig config =
                OCCRetryConfig.builder().maxRetries(3).baseDelayMs(50).maxDelayMs(200).build();

        Thread.currentThread().interrupt();

        SQLException thrown =
                assertThrows(
                        SQLException.class,
                        () ->
                                OCCRetry.execute(
                                        conn,
                                        config,
                                        c -> {
                                            throw new SQLException("conflict", "OC000");
                                        }));

        assertTrue(thrown.getMessage().contains("interrupted"));
        assertTrue(Thread.interrupted());
    }

    // --- DataSource.getConnection() failure test ---

    @Test
    void executeDataSource_getConnectionThrows_propagatesImmediately() throws SQLException {
        DataSource ds = mock(DataSource.class);
        SQLException poolExhausted = new SQLException("pool exhausted", "08001");
        when(ds.getConnection()).thenThrow(poolExhausted);

        OCCRetryConfig config =
                OCCRetryConfig.builder().maxRetries(3).baseDelayMs(1).maxDelayMs(10).build();

        SQLException thrown =
                assertThrows(SQLException.class, () -> OCCRetry.execute(ds, config, c -> "never"));

        assertEquals(poolExhausted, thrown);
    }

    // --- rollback failure tests ---

    @Test
    void executeConnection_rollbackFailure_suppressedOnOriginalException() throws SQLException {
        Connection conn = mock(Connection.class);
        SQLException rollbackEx = new SQLException("connection reset");
        org.mockito.Mockito.doThrow(rollbackEx).when(conn).rollback();

        SQLException original = new SQLException("constraint", "23505");
        OCCRetryConfig config = OCCRetryConfig.builder().baseDelayMs(1).maxDelayMs(10).build();

        SQLException thrown =
                assertThrows(
                        SQLException.class,
                        () ->
                                OCCRetry.execute(
                                        conn,
                                        config,
                                        c -> {
                                            throw original;
                                        }));

        assertEquals(original, thrown);
        assertEquals(1, thrown.getSuppressed().length);
        assertEquals(rollbackEx, thrown.getSuppressed()[0]);
    }

    @Test
    void executeConnection_rollbackFailure_onOCC_continuesRetry() throws SQLException {
        Connection conn = mock(Connection.class);
        SQLException rollbackEx = new SQLException("connection reset");
        org.mockito.Mockito.doThrow(rollbackEx).when(conn).rollback();

        AtomicInteger attempts = new AtomicInteger();
        OCCRetryConfig config =
                OCCRetryConfig.builder().maxRetries(1).baseDelayMs(1).maxDelayMs(10).build();

        SQLException thrown =
                assertThrows(
                        SQLException.class,
                        () ->
                                OCCRetry.execute(
                                        conn,
                                        config,
                                        c -> {
                                            attempts.incrementAndGet();
                                            throw new SQLException("conflict", "OC000");
                                        }));

        assertEquals(2, attempts.get());
        assertEquals("OC000", thrown.getSQLState());
        assertTrue(thrown.getSuppressed().length > 0);
    }

    // --- calculateBackoff tests ---

    @Test
    void calculateBackoff_firstAttempt_returnsBaseDelay() {
        OCCRetryConfig config =
                OCCRetryConfig.builder()
                        .baseDelayMs(100)
                        .maxDelayMs(5000)
                        .multiplier(2.0)
                        .jitterFactor(0.0)
                        .build();

        long backoff = OCCRetry.calculateBackoff(config, 0);
        assertEquals(100, backoff);
    }

    @Test
    void calculateBackoff_secondAttempt_appliesMultiplier() {
        OCCRetryConfig config =
                OCCRetryConfig.builder()
                        .baseDelayMs(100)
                        .maxDelayMs(5000)
                        .multiplier(2.0)
                        .jitterFactor(0.0)
                        .build();

        long backoff = OCCRetry.calculateBackoff(config, 1);
        assertEquals(200, backoff);
    }

    @Test
    void calculateBackoff_cappedAtMaxDelay() {
        OCCRetryConfig config =
                OCCRetryConfig.builder()
                        .baseDelayMs(100)
                        .maxDelayMs(500)
                        .multiplier(2.0)
                        .jitterFactor(0.0)
                        .build();

        long backoff = OCCRetry.calculateBackoff(config, 10);
        assertEquals(500, backoff);
    }

    @Test
    void calculateBackoff_withJitter_doesNotExceedMaxPlusJitter() {
        OCCRetryConfig config =
                OCCRetryConfig.builder()
                        .baseDelayMs(100)
                        .maxDelayMs(500)
                        .multiplier(2.0)
                        .jitterFactor(1.0)
                        .build();

        for (int i = 0; i < 100; i++) {
            long backoff = OCCRetry.calculateBackoff(config, 10);
            assertTrue(backoff >= 500, "backoff should be at least maxDelay");
            assertTrue(backoff <= 1000, "backoff should not exceed maxDelay + maxDelay*jitter");
        }
    }
}
