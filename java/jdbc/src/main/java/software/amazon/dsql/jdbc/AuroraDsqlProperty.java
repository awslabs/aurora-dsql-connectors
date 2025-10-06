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

import java.sql.DriverPropertyInfo;
import java.util.Properties;
import javax.annotation.Nonnull;
import javax.annotation.Nullable;

/**
 * Represents a connection property for the Aurora DSQL JDBC driver.
 *
 * <p>This class extends {@link DriverPropertyInfo} to provide additional functionality for managing
 * connection properties.
 *
 * @see DriverPropertyInfo
 * @since 1.0.0
 */
public class AuroraDsqlProperty extends DriverPropertyInfo {

    /** The default value for this property, or {@code null} if no default is defined. */
    public final @Nullable String defaultValue;

    /**
     * Creates an optional property which accepts any value.
     *
     * @param name the property name
     * @param defaultValue the default value, or {@code null} if no default
     * @param description the property description, or {@code null} if no description
     */
    public AuroraDsqlProperty(
            @Nonnull final String name,
            @Nullable final String defaultValue,
            @Nullable final String description) {
        this(name, defaultValue, description, false);
    }

    /**
     * Creates a property which accepts any value.
     *
     * @param name the property name
     * @param defaultValue the default value, or {@code null} if no default
     * @param description the property description, or {@code null} if no description
     * @param required whether this property is required for connections
     */
    public AuroraDsqlProperty(
            @Nonnull final String name,
            @Nullable final String defaultValue,
            @Nullable final String description,
            final boolean required) {
        this(name, defaultValue, description, required, null);
    }

    /**
     * Creates a property.
     *
     * @param name the property name
     * @param defaultValue the default value, or {@code null} if no default
     * @param description the property description, or {@code null} if no description
     * @param required whether this property is required for connections
     * @param choices valid choices for this property, or {@code null} if any value is allowed
     */
    public AuroraDsqlProperty(
            @Nonnull final String name,
            @Nullable final String defaultValue,
            @Nullable final String description,
            final boolean required,
            @Nullable final String[] choices) {
        super(name, null);
        this.defaultValue = defaultValue;
        this.required = required;
        this.description = description;
        this.choices = choices;
    }

    /**
     * Retrieves the property value from the given {@link Properties}, or the default value if not
     * set.
     *
     * @param properties the {@link Properties} to query
     * @return the property value, or the default value if not set, or {@code null} if neither
     *     exists
     */
    public @Nullable String getOrDefault(Properties properties) {
        return properties.getProperty(this.name, this.defaultValue);
    }

    /**
     * Retrieves the property value from the given {@link Properties}.
     *
     * @param properties the {@link Properties} to query
     * @return the property value, or {@code null} if not set
     */
    public @Nullable String get(Properties properties) {
        return properties.getProperty(this.name);
    }

    /**
     * Sets the property value in the given {@link Properties}.
     *
     * <p>If the value is {@code null}, the property is removed from the {@link Properties} object.
     *
     * @param properties the {@link Properties} to modify
     * @param value the value to set, or {@code null} to remove the property
     */
    public void set(Properties properties, @Nullable String value) {
        if (value == null) {
            properties.remove(name);
        } else {
            properties.setProperty(name, value);
        }
    }

    /**
     * Retrieves the property value as an integer.
     *
     * <p>If the property is not set and no default value exists, returns {@code 0}.
     *
     * @param properties the {@link Properties} to query
     * @return the property value as an integer, or {@code 0} if not set and no default exists
     * @throws NumberFormatException if the property value cannot be parsed as an integer
     */
    public int getInteger(final Properties properties) {
        final Object value = properties.get(name);
        if (value == null && defaultValue == null) {
            return 0;
        }
        if (value instanceof Integer) {
            return (Integer) value;
        }
        return Integer.parseInt(properties.getProperty(name, defaultValue));
    }

    /**
     * Retrieves the property value as a long.
     *
     * <p>If the property is not set and no default value exists, returns {@code 0L}.
     *
     * @param properties the {@link Properties} to query
     * @return the property value as a long, or {@code 0L} if not set and no default exists
     * @throws NumberFormatException if the property value cannot be parsed as a long
     */
    public long getLong(final Properties properties) {
        final Object value = properties.get(name);
        if (value == null && defaultValue == null) {
            return 0L;
        }
        if (value instanceof Long) {
            return (Long) value;
        }
        return Long.parseLong(properties.getProperty(name, defaultValue));
    }

    /**
     * Converts this property to a {@link DriverPropertyInfo} instance.
     *
     * <p>Creates a {@link DriverPropertyInfo} object containing this property's metadata (name,
     * description, etc.) along with the current value from the provided {@link Properties} object.
     * If the property is not set in the {@link Properties} object, the value will be {@code null}.
     *
     * @param properties the {@link Properties} to query for the current value
     * @return a {@link DriverPropertyInfo} instance with property metadata and provided property
     *     value
     */
    public DriverPropertyInfo toDriverPropertyInfo(final Properties properties) {
        final DriverPropertyInfo propertyInfo = new DriverPropertyInfo(name, get(properties));
        propertyInfo.required = required;
        propertyInfo.description = description;
        propertyInfo.choices = choices;
        return propertyInfo;
    }
}
