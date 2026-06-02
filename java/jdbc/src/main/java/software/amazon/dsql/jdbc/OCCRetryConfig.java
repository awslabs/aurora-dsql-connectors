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

/**
 * Immutable configuration for OCC (Optimistic Concurrency Control) retry behavior.
 *
 * <p>Use {@link #defaults()} for standard settings or {@link #builder()} for custom configuration.
 *
 * <pre>{@code
 * // Use defaults (see DEFAULT_* constants)
 * OCCRetryConfig config = OCCRetryConfig.defaults();
 *
 * // Custom configuration
 * OCCRetryConfig config = OCCRetryConfig.builder()
 *     .maxRetries(5)
 *     .baseDelayMs(200)
 *     .build();
 * }</pre>
 */
public final class OCCRetryConfig {

    private static final int DEFAULT_MAX_RETRIES = 3;
    private static final long DEFAULT_BASE_DELAY_MS = 100;
    private static final long DEFAULT_MAX_DELAY_MS = 5000;
    private static final double DEFAULT_MULTIPLIER = 2.0;
    private static final double DEFAULT_JITTER_FACTOR = 0.25;

    private final int maxRetries;
    private final long baseDelayMs;
    private final long maxDelayMs;
    private final double multiplier;
    private final double jitterFactor;

    private static final OCCRetryConfig DEFAULT = new Builder().build();

    private OCCRetryConfig(Builder builder) {
        this.maxRetries = builder.maxRetries;
        this.baseDelayMs = builder.baseDelayMs;
        this.maxDelayMs = builder.maxDelayMs;
        this.multiplier = builder.multiplier;
        this.jitterFactor = builder.jitterFactor;
    }

    /**
     * Returns a configuration with default values.
     *
     * @return default configuration
     */
    public static OCCRetryConfig defaults() {
        return DEFAULT;
    }

    /**
     * Returns a new builder for custom configuration.
     *
     * @return a new builder instance
     */
    public static Builder builder() {
        return new Builder();
    }

    public int getMaxRetries() {
        return maxRetries;
    }

    public long getBaseDelayMs() {
        return baseDelayMs;
    }

    public long getMaxDelayMs() {
        return maxDelayMs;
    }

    public double getMultiplier() {
        return multiplier;
    }

    public double getJitterFactor() {
        return jitterFactor;
    }

    /**
     * Builder for {@link OCCRetryConfig}.
     *
     * <p>Validates all parameters at {@link #build()} time.
     */
    public static final class Builder {

        private int maxRetries = DEFAULT_MAX_RETRIES;
        private long baseDelayMs = DEFAULT_BASE_DELAY_MS;
        private long maxDelayMs = DEFAULT_MAX_DELAY_MS;
        private double multiplier = DEFAULT_MULTIPLIER;
        private double jitterFactor = DEFAULT_JITTER_FACTOR;

        private Builder() {}

        /**
         * Maximum number of retry attempts. Range: 0–100. A value of 0 means execute once with no
         * retry.
         */
        public Builder maxRetries(int maxRetries) {
            this.maxRetries = maxRetries;
            return this;
        }

        /** Base delay in milliseconds before the first retry. Must be greater than 0. */
        public Builder baseDelayMs(long baseDelayMs) {
            this.baseDelayMs = baseDelayMs;
            return this;
        }

        /** Maximum delay in milliseconds between retries. Must be >= baseDelayMs. */
        public Builder maxDelayMs(long maxDelayMs) {
            this.maxDelayMs = maxDelayMs;
            return this;
        }

        /** Exponential backoff multiplier. Must be >= 1.0. */
        public Builder multiplier(double multiplier) {
            this.multiplier = multiplier;
            return this;
        }

        /** Jitter factor applied to backoff delay. Range: 0.0–1.0. */
        public Builder jitterFactor(double jitterFactor) {
            this.jitterFactor = jitterFactor;
            return this;
        }

        /**
         * Builds the configuration, validating all parameters.
         *
         * @return validated configuration
         * @throws IllegalArgumentException if any parameter is out of range
         */
        public OCCRetryConfig build() {
            if (maxRetries < 0 || maxRetries > 100) {
                throw new IllegalArgumentException(
                        "maxRetries must be between 0 and 100, got: " + maxRetries);
            }
            if (baseDelayMs <= 0) {
                throw new IllegalArgumentException(
                        "baseDelayMs must be greater than 0, got: " + baseDelayMs);
            }
            if (maxDelayMs < baseDelayMs) {
                throw new IllegalArgumentException(
                        "maxDelayMs must be >= baseDelayMs, got maxDelayMs="
                                + maxDelayMs
                                + " baseDelayMs="
                                + baseDelayMs);
            }
            if (!Double.isFinite(multiplier) || multiplier < 1.0) {
                throw new IllegalArgumentException("multiplier must be >= 1.0, got: " + multiplier);
            }
            if (!Double.isFinite(jitterFactor) || jitterFactor < 0.0 || jitterFactor > 1.0) {
                throw new IllegalArgumentException(
                        "jitterFactor must be between 0.0 and 1.0, got: " + jitterFactor);
            }
            return new OCCRetryConfig(this);
        }
    }
}
