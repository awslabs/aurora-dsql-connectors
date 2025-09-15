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

import java.util.Map;
import java.util.Properties;
import java.util.logging.Logger;
import javax.annotation.Nonnull;

public final class PropertyUtils {
    private static final Logger LOGGER = Logger.getLogger(PropertyUtils.class.getName());

    private PropertyUtils() {
        throw new UnsupportedOperationException(
                "This is a utility class and cannot be instantiated");
    }

    public static @Nonnull Properties copyProperties(final Properties props) {
        final Properties copy = new Properties();

        if (props == null) {
            return copy;
        }

        return addProperties(copy, props);
    }

    public static @Nonnull Properties addProperties(
            final Properties dest, final Properties propsToAdd) {

        if (dest == null) {
            throw new IllegalArgumentException("The 'dest' parameter cannot be null.");
        }

        if (propsToAdd == null) {
            return dest;
        }

        for (final Map.Entry<Object, Object> entry : propsToAdd.entrySet()) {
            if (entry.getKey() == null || entry.getValue() == null) {
                throw new IllegalArgumentException(
                        "The key and value of the entry cannot be null.");
            }
            String key = entry.getKey().toString();
            String value = sanitizePropertyValue(entry.getValue().toString());
            dest.setProperty(key, value);
        }
        return dest;
    }

    /**
     * Sanitizes a property value by removing quotes and trimming whitespace
     *
     * @param value the value to sanitize
     * @return sanitized value
     */
    public static String sanitizePropertyValue(String value) {
        if (value == null) {
            return null;
        }

        String trimmed = value.trim();

        // Remove surrounding quotes if present
        if (trimmed.length() >= 2) {
            if ((trimmed.startsWith("\"") && trimmed.endsWith("\""))
                    || (trimmed.startsWith("'") && trimmed.endsWith("'"))) {
                String unquoted = trimmed.substring(1, trimmed.length() - 1);
                LOGGER.fine(
                        () ->
                                String.format(
                                        "Sanitized property value from '%s' to '%s'",
                                        value, unquoted));
                return unquoted;
            }
        }

        return trimmed;
    }

    /**
     * Gets a property value and sanitizes it
     *
     * @param props Properties object
     * @param key Property key
     * @return sanitized value or null if not found
     */
    public static String getProperty(Properties props, String key) {
        String value = props.getProperty(key);
        return sanitizePropertyValue(value);
    }
}
