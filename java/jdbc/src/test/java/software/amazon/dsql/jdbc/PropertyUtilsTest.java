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
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNotSame;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.Properties;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

class PropertyUtilsTest {

    private Properties properties;

    @BeforeEach
    void setUp() {
        properties = new Properties();
        properties.setProperty("key1", "value1");
        properties.setProperty("key2", "value2");
    }

    @Test
    void testCopyPropertiesWithValidProperties() {
        // Act
        Properties result = PropertyUtils.copyProperties(properties);

        // Assert
        assertNotNull(result);
        assertEquals("value1", result.getProperty("key1"));
        assertEquals("value2", result.getProperty("key2"));
        assertEquals(2, result.size());
    }

    @Test
    void testCopyPropertiesWithNullProperties() {
        // Act
        Properties result = PropertyUtils.copyProperties(null);

        // Assert
        assertNotNull(result);
        assertTrue(result.isEmpty());
    }

    @Test
    void testCopyPropertiesWithEmptyProperties() {
        // Arrange
        Properties emptyProps = new Properties();

        // Act
        Properties result = PropertyUtils.copyProperties(emptyProps);

        // Assert
        assertNotNull(result);
        assertTrue(result.isEmpty());
    }

    @Test
    void testAddPropertiesWithValidInputs() {
        // Arrange
        Properties destination = new Properties();
        destination.setProperty("existing", "value");

        // Act
        Properties result = PropertyUtils.addProperties(destination, properties);

        // Assert
        assertSame(destination, result); // Should return the same instance
        assertEquals("value", result.getProperty("existing"));
        assertEquals("value1", result.getProperty("key1"));
        assertEquals("value2", result.getProperty("key2"));
        assertEquals(3, result.size());
    }

    @Test
    void testAddPropertiesWithNullDestination() {
        // Act & Assert
        assertThrows(IllegalArgumentException.class, () -> 
            PropertyUtils.addProperties(null, properties)
        );
    }

    @Test
    void testAddPropertiesWithNullPropsToAdd() {
        // Arrange
        Properties destination = new Properties();
        destination.setProperty("existing", "value");

        // Act
        Properties result = PropertyUtils.addProperties(destination, null);

        // Assert
        assertSame(destination, result);
        assertEquals("value", result.getProperty("existing"));
        assertEquals(1, result.size());
    }

    @Test
    void testAddPropertiesOverwritesExistingKeys() {
        // Arrange
        Properties destination = new Properties();
        destination.setProperty("key1", "old_value");
        destination.setProperty("unique", "unique_value");

        // Act
        Properties result = PropertyUtils.addProperties(destination, properties);

        // Assert
        assertEquals("value1", result.getProperty("key1")); // Should be overwritten
        assertEquals("unique_value", result.getProperty("unique"));
        assertEquals("value2", result.getProperty("key2"));
        assertEquals(3, result.size());
    }

    @Test
    void testAddPropertiesWithNullKeyThrowsException() {
        // Arrange
        Properties destination = new Properties();
        Properties propsWithNullKey = new Properties() {
            @Override
            public java.util.Set<java.util.Map.Entry<Object, Object>> entrySet() {
                java.util.Set<java.util.Map.Entry<Object, Object>> set = new java.util.HashSet<>();
                set.add(new java.util.AbstractMap.SimpleEntry<>(null, "value"));
                return set;
            }
        };

        // Act & Assert
        assertThrows(IllegalArgumentException.class, () -> 
            PropertyUtils.addProperties(destination, propsWithNullKey)
        );
    }

    @Test
    void testAddPropertiesWithNullValueThrowsException() {
        // Arrange
        Properties destination = new Properties();
        Properties propsWithNullValue = new Properties() {
            @Override
            public java.util.Set<java.util.Map.Entry<Object, Object>> entrySet() {
                java.util.Set<java.util.Map.Entry<Object, Object>> set = new java.util.HashSet<>();
                set.add(new java.util.AbstractMap.SimpleEntry<>("key", null));
                return set;
            }
        };

        // Act & Assert
        assertThrows(IllegalArgumentException.class, () -> 
            PropertyUtils.addProperties(destination, propsWithNullValue)
        );
    }

    @Test
    void testConstructorThrowsException() {
        // Act & Assert
        Exception exception = assertThrows(Exception.class, () -> {
            // Use reflection to access private constructor
            java.lang.reflect.Constructor<PropertyUtils> constructor = 
                PropertyUtils.class.getDeclaredConstructor();
            constructor.setAccessible(true);
            constructor.newInstance();
        });
        
        // The actual exception will be wrapped in InvocationTargetException
        assertTrue(exception.getCause() instanceof UnsupportedOperationException);
    }

    @Test
    void testCopyPropertiesCreatesNewInstance() {
        // Act
        Properties result = PropertyUtils.copyProperties(properties);

        // Assert
        assertNotSame(properties, result); // Should be different instances
        
        // Modify original and verify copy is unaffected
        properties.setProperty("key1", "modified");
        assertEquals("value1", result.getProperty("key1")); // Copy should be unchanged
    }

    @Test
    void testAddPropertiesWithEmptyPropsToAdd() {
        // Arrange
        Properties destination = new Properties();
        destination.setProperty("existing", "value");
        Properties emptyProps = new Properties();

        // Act
        Properties result = PropertyUtils.addProperties(destination, emptyProps);

        // Assert
        assertSame(destination, result);
        assertEquals("value", result.getProperty("existing"));
        assertEquals(1, result.size());
    }


    @Test
    void testSanitizePropertyValue() {
        // Test null handling
        assertNull(PropertyUtils.sanitizePropertyValue(null));

        // Test quoted values
        assertEquals("value", PropertyUtils.sanitizePropertyValue("\"value\""));
        assertEquals("value", PropertyUtils.sanitizePropertyValue("'value'"));

        // Test whitespace and quotes
        assertEquals("value", PropertyUtils.sanitizePropertyValue("  \"value\"  "));
        assertEquals("value", PropertyUtils.sanitizePropertyValue("  'value'  "));

        // Test unquoted values
        assertEquals("value", PropertyUtils.sanitizePropertyValue("value"));
        assertEquals("value", PropertyUtils.sanitizePropertyValue("  value  "));

        // Test empty values
        assertEquals("", PropertyUtils.sanitizePropertyValue(""));
        assertEquals("", PropertyUtils.sanitizePropertyValue("\"\""));
        assertEquals("", PropertyUtils.sanitizePropertyValue("''"));

        // Test single quotes
        assertEquals("value\"", PropertyUtils.sanitizePropertyValue("value\""));
        assertEquals("\"value", PropertyUtils.sanitizePropertyValue("\"value"));
    }

    @Test
    void testAddProperties() {
        // Arrange
        Properties source = new Properties();
        source.setProperty("region", "\"us-east-1\"");
        source.setProperty("cluster-id", "'test-cluster'");
        source.setProperty("normal", "value");

        Properties dest = new Properties();

        // Act
        Properties result = PropertyUtils.addProperties(dest, source);

        // Assert
        assertEquals("us-east-1", result.getProperty("region"));
        assertEquals("test-cluster", result.getProperty("cluster-id"));
        assertEquals("value", result.getProperty("normal"));
    }

    @Test
    void testCopyProperties() {
        // Arrange
        Properties source = new Properties();
        source.setProperty("region", "\"us-east-1\"");
        source.setProperty("cluster-id", "'test-cluster'");

        // Act
        Properties result = PropertyUtils.copyProperties(source);

        // Assert
        assertEquals("us-east-1", result.getProperty("region"));
        assertEquals("test-cluster", result.getProperty("cluster-id"));
    }

    @Test
    void testGetProperty() {
        // Arrange
        Properties props = new Properties();
        props.setProperty("quoted", "\"value\"");
        props.setProperty("unquoted", "value");

        // Act & Assert
        assertEquals("value", PropertyUtils.getProperty(props, "quoted"));
        assertEquals("value", PropertyUtils.getProperty(props, "unquoted"));
        assertNull(PropertyUtils.getProperty(props, "nonexistent"));
    }

    @Test
    void testNullHandling() {
        // Test null source properties
        assertNotNull(PropertyUtils.copyProperties(null));

        // Test null destination properties
        assertThrows(IllegalArgumentException.class, () ->
                PropertyUtils.addProperties(null, new Properties()));
    }
}
