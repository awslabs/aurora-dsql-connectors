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

import javax.annotation.Nonnull;
import java.util.function.Supplier;
import java.util.logging.Level;
import java.util.logging.Logger;

/**
 * Lazy logging wrapper that only evaluates log messages when the level is enabled.
 */
public class LazyLogger {
    private final Logger logger;

    public LazyLogger(@Nonnull final String loggerName) {
        this.logger = Logger.getLogger(loggerName);
    }

    public void fine(@Nonnull final Supplier<String> messageSupplier) {
        if (logger.isLoggable(Level.FINE)) {
            logger.fine(messageSupplier.get());
        }
    }

    public void warning(@Nonnull final String message) {
        logger.warning(message);
    }

    public void severe(@Nonnull final String message) {
        logger.severe(message);
    }
}
