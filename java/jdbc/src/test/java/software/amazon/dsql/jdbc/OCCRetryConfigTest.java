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

import org.junit.jupiter.api.Test;

class OCCRetryConfigTest {

    @Test
    void defaults_returnsExpectedValues() {
        OCCRetryConfig config = OCCRetryConfig.defaults();

        assertEquals(3, config.getMaxRetries());
        assertEquals(100, config.getBaseDelayMs());
        assertEquals(5000, config.getMaxDelayMs());
        assertEquals(2.0, config.getMultiplier());
        assertEquals(0.25, config.getJitterFactor());
    }

    @Test
    void builder_customValues() {
        OCCRetryConfig config =
                OCCRetryConfig.builder()
                        .maxRetries(5)
                        .baseDelayMs(200)
                        .maxDelayMs(10000)
                        .multiplier(3.0)
                        .jitterFactor(0.5)
                        .build();

        assertEquals(5, config.getMaxRetries());
        assertEquals(200, config.getBaseDelayMs());
        assertEquals(10000, config.getMaxDelayMs());
        assertEquals(3.0, config.getMultiplier());
        assertEquals(0.5, config.getJitterFactor());
    }

    @Test
    void builder_maxRetriesZero_allowed() {
        OCCRetryConfig config = OCCRetryConfig.builder().maxRetries(0).build();
        assertEquals(0, config.getMaxRetries());
    }

    @Test
    void builder_maxRetriesNegative_throws() {
        assertThrows(
                IllegalArgumentException.class,
                () -> OCCRetryConfig.builder().maxRetries(-1).build());
    }

    @Test
    void builder_maxRetriesExceeds100_throws() {
        assertThrows(
                IllegalArgumentException.class,
                () -> OCCRetryConfig.builder().maxRetries(101).build());
    }

    @Test
    void builder_baseDelayMsZero_throws() {
        assertThrows(
                IllegalArgumentException.class,
                () -> OCCRetryConfig.builder().baseDelayMs(0).build());
    }

    @Test
    void builder_baseDelayMsNegative_throws() {
        assertThrows(
                IllegalArgumentException.class,
                () -> OCCRetryConfig.builder().baseDelayMs(-1).build());
    }

    @Test
    void builder_maxDelayLessThanBase_throws() {
        assertThrows(
                IllegalArgumentException.class,
                () -> OCCRetryConfig.builder().baseDelayMs(200).maxDelayMs(50).build());
    }

    @Test
    void builder_multiplierLessThanOne_throws() {
        assertThrows(
                IllegalArgumentException.class,
                () -> OCCRetryConfig.builder().multiplier(0.5).build());
    }

    @Test
    void builder_jitterFactorNegative_throws() {
        assertThrows(
                IllegalArgumentException.class,
                () -> OCCRetryConfig.builder().jitterFactor(-0.1).build());
    }

    @Test
    void builder_jitterFactorAboveOne_throws() {
        assertThrows(
                IllegalArgumentException.class,
                () -> OCCRetryConfig.builder().jitterFactor(1.1).build());
    }

    @Test
    void builder_jitterFactorBoundaries_allowed() {
        OCCRetryConfig zero = OCCRetryConfig.builder().jitterFactor(0.0).build();
        assertEquals(0.0, zero.getJitterFactor());

        OCCRetryConfig one = OCCRetryConfig.builder().jitterFactor(1.0).build();
        assertEquals(1.0, one.getJitterFactor());
    }

    @Test
    void builder_multiplierNaN_throws() {
        assertThrows(
                IllegalArgumentException.class,
                () -> OCCRetryConfig.builder().multiplier(Double.NaN).build());
    }

    @Test
    void builder_multiplierInfinity_throws() {
        assertThrows(
                IllegalArgumentException.class,
                () -> OCCRetryConfig.builder().multiplier(Double.POSITIVE_INFINITY).build());
    }

    @Test
    void builder_jitterFactorNaN_throws() {
        assertThrows(
                IllegalArgumentException.class,
                () -> OCCRetryConfig.builder().jitterFactor(Double.NaN).build());
    }
}
