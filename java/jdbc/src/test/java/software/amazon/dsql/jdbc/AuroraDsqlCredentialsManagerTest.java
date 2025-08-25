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

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.AfterEach;

import software.amazon.awssdk.auth.credentials.AwsCredentialsProvider;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNotSame;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.mock;

class AuroraDsqlCredentialsManagerTest {

    @BeforeEach
    void setUp() {
        // Reset to default provider before each test
        AuroraDsqlCredentialsManager.resetProvider();
    }

    @AfterEach
    void tearDown() {
        // Reset to default provider after each test
        AuroraDsqlCredentialsManager.resetProvider();
    }

    @Test
    void testGetProviderReturnsDefaultProvider() {
        // Act
        AwsCredentialsProvider result = AuroraDsqlCredentialsManager.getProvider();

        // Assert
        assertNotNull(result);
        // Should be DefaultCredentialsProvider (we can't easily test the exact type without more setup)
    }

    @Test
    void testSetAndGetCustomProvider() {
        // Arrange
        AwsCredentialsProvider customProvider = mock(AwsCredentialsProvider.class);

        // Act
        AuroraDsqlCredentialsManager.setProvider(customProvider);
        AwsCredentialsProvider result = AuroraDsqlCredentialsManager.getProvider();

        // Assert
        assertSame(customProvider, result);
    }

    @Test
    void testSetNullProvider() {
        // Arrange
        AwsCredentialsProvider customProvider = mock(AwsCredentialsProvider.class);
        AuroraDsqlCredentialsManager.setProvider(customProvider);

        // Act
        AuroraDsqlCredentialsManager.setProvider(null);
        AwsCredentialsProvider result = AuroraDsqlCredentialsManager.getProvider();

        // Assert
        assertNull(result);
    }

    @Test
    void testResetProvider() {
        // Arrange
        AwsCredentialsProvider customProvider = mock(AwsCredentialsProvider.class);
        AuroraDsqlCredentialsManager.setProvider(customProvider);

        // Act
        AuroraDsqlCredentialsManager.resetProvider();
        AwsCredentialsProvider result = AuroraDsqlCredentialsManager.getProvider();

        // Assert
        assertNotNull(result);
        assertNotSame(customProvider, result);
        // Should be back to DefaultCredentialsProvider
    }

    @Test
    void testSetProviderOverridesPrevious() {
        // Arrange
        AwsCredentialsProvider provider1 = mock(AwsCredentialsProvider.class);
        AwsCredentialsProvider provider2 = mock(AwsCredentialsProvider.class);

        // Act
        AuroraDsqlCredentialsManager.setProvider(provider1);
        AwsCredentialsProvider firstResult = AuroraDsqlCredentialsManager.getProvider();
        
        AuroraDsqlCredentialsManager.setProvider(provider2);
        AwsCredentialsProvider secondResult = AuroraDsqlCredentialsManager.getProvider();

        // Assert
        assertSame(provider1, firstResult);
        assertSame(provider2, secondResult);
        assertNotSame(firstResult, secondResult);
    }

    @Test
    void testThreadSafety() throws InterruptedException {
        // Arrange
        AwsCredentialsProvider provider1 = mock(AwsCredentialsProvider.class);
        AwsCredentialsProvider provider2 = mock(AwsCredentialsProvider.class);

        // Act
        Thread thread1 = new Thread(() -> {
            AuroraDsqlCredentialsManager.setProvider(provider1);
            try {
                Thread.sleep(10);
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            }
        });

        Thread thread2 = new Thread(() -> {
            AuroraDsqlCredentialsManager.setProvider(provider2);
            try {
                Thread.sleep(10);
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            }
        });

        thread1.start();
        thread2.start();
        thread1.join();
        thread2.join();

        // Assert
        AwsCredentialsProvider finalProvider = AuroraDsqlCredentialsManager.getProvider();
        assertTrue(finalProvider == provider1 || finalProvider == provider2);
    }

    @Test
    void testMultipleGetCallsReturnSameInstance() {
        // Arrange
        AwsCredentialsProvider customProvider = mock(AwsCredentialsProvider.class);
        AuroraDsqlCredentialsManager.setProvider(customProvider);

        // Act
        AwsCredentialsProvider result1 = AuroraDsqlCredentialsManager.getProvider();
        AwsCredentialsProvider result2 = AuroraDsqlCredentialsManager.getProvider();

        // Assert
        assertSame(result1, result2);
        assertSame(customProvider, result1);
    }

    @Test
    void testConstructorThrowsException() {
        // Act & Assert
        Exception exception = assertThrows(Exception.class, () -> {
            // Use reflection to access private constructor
            java.lang.reflect.Constructor<AuroraDsqlCredentialsManager> constructor = 
                AuroraDsqlCredentialsManager.class.getDeclaredConstructor();
            constructor.setAccessible(true);
            constructor.newInstance();
        });
        
        // The actual exception will be wrapped in InvocationTargetException
        assertTrue(exception.getCause() instanceof UnsupportedOperationException);
    }

    @Test
    void testResetAfterMultipleChanges() {
        // Arrange
        AwsCredentialsProvider provider1 = mock(AwsCredentialsProvider.class);
        AwsCredentialsProvider provider2 = mock(AwsCredentialsProvider.class);

        // Act
        AuroraDsqlCredentialsManager.setProvider(provider1);
        AuroraDsqlCredentialsManager.setProvider(provider2);
        AuroraDsqlCredentialsManager.setProvider(null);
        AuroraDsqlCredentialsManager.resetProvider();
        
        AwsCredentialsProvider result = AuroraDsqlCredentialsManager.getProvider();

        // Assert
        assertNotNull(result);
        assertNotSame(provider1, result);
        assertNotSame(provider2, result);
    }
}
