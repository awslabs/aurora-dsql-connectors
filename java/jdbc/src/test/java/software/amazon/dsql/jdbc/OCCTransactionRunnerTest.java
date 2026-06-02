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
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

import java.sql.Connection;
import java.sql.SQLException;
import java.util.concurrent.atomic.AtomicInteger;
import javax.sql.DataSource;
import org.junit.jupiter.api.Test;

class OCCTransactionRunnerTest {

    @Test
    void run_delegatesToOCCRetryExecute() throws SQLException {
        DataSource ds = mock(DataSource.class);
        Connection conn = mock(Connection.class);
        when(ds.getConnection()).thenReturn(conn);

        OCCTransactionRunner runner = OCCTransactionRunner.create(ds);
        String result = runner.run(c -> "value");

        assertEquals("value", result);
        verify(conn).setAutoCommit(false);
        verify(conn).commit();
    }

    @Test
    void runVoid_delegatesToOCCRetryExecute() throws SQLException {
        DataSource ds = mock(DataSource.class);
        Connection conn = mock(Connection.class);
        when(ds.getConnection()).thenReturn(conn);

        OCCTransactionRunner runner = OCCTransactionRunner.create(ds);
        AtomicInteger called = new AtomicInteger();
        runner.runVoid(c -> called.incrementAndGet());

        assertEquals(1, called.get());
        verify(conn).commit();
    }

    @Test
    void run_retriesOnOCC() throws SQLException {
        DataSource ds = mock(DataSource.class);
        Connection conn = mock(Connection.class);
        when(ds.getConnection()).thenReturn(conn);

        OCCRetryConfig config =
                OCCRetryConfig.builder().maxRetries(2).baseDelayMs(1).maxDelayMs(10).build();
        OCCTransactionRunner runner = OCCTransactionRunner.create(ds, config);

        AtomicInteger attempts = new AtomicInteger();
        String result =
                runner.run(
                        c -> {
                            if (attempts.getAndIncrement() == 0) {
                                throw new SQLException("conflict", "OC000");
                            }
                            return "recovered";
                        });

        assertEquals("recovered", result);
        assertEquals(2, attempts.get());
    }

    @Test
    void create_nullDataSource_throws() {
        assertThrows(NullPointerException.class, () -> OCCTransactionRunner.create(null));
    }

    @Test
    void create_nullConfig_throws() {
        DataSource ds = mock(DataSource.class);
        assertThrows(NullPointerException.class, () -> OCCTransactionRunner.create(ds, null));
    }
}
