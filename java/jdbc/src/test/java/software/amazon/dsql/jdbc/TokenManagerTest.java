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

import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.mockito.MockedStatic;
import software.amazon.awssdk.auth.credentials.AwsCredentialsProvider;
import software.amazon.awssdk.regions.Region;
import software.amazon.awssdk.services.dsql.DsqlUtilities;

import java.sql.SQLException;
import java.time.Duration;
import java.util.function.Consumer;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.*;

class TokenManagerTest {

    private static final String TEST_HOSTNAME = "test-cluster.dsql.us-east-1.on.aws";
    private static final Region TEST_REGION = Region.US_EAST_1;
    private static final String TEST_USER = "testuser";
    private static final String ADMIN_USER = "admin";
    private static final Duration TEST_DURATION = Duration.ofMinutes(30);

    private AwsCredentialsProvider mockCredentialsProvider;
    private DsqlUtilities mockDsqlUtilities;
    private DsqlUtilities.Builder mockBuilder;

    @BeforeEach
    void setUp() {
        mockCredentialsProvider = mock(AwsCredentialsProvider.class);
        mockDsqlUtilities = mock(DsqlUtilities.class);
        mockBuilder = mock(DsqlUtilities.Builder.class);
        
        // Clear cache before each test
        TokenManager.clearCache();
    }

    @AfterEach
    void tearDown() {
        // Clear cache after each test
        TokenManager.clearCache();
    }

    @Test
    void testGetToken_FirstCall_GeneratesNewToken() throws SQLException {
        // Arrange
        String expectedToken = "test-token";
        setupMockDsqlUtilities(expectedToken);

        try (MockedStatic<DsqlUtilities> dsqlUtilitiesMock = mockStatic(DsqlUtilities.class)) {
            dsqlUtilitiesMock.when(DsqlUtilities::builder).thenReturn(mockBuilder);

            // Act
            String token = TokenManager.getToken(TEST_HOSTNAME, TEST_REGION, TEST_USER, mockCredentialsProvider, TEST_DURATION);

            // Assert
            assertEquals(expectedToken, token);
            verify(mockDsqlUtilities).generateDbConnectAuthToken(any(Consumer.class));
            verify(mockDsqlUtilities, never()).generateDbConnectAdminAuthToken(any(Consumer.class));
        }
    }

    @Test
    void testGetToken_AdminUser_UsesAdminTokenGenerator() throws SQLException {
        // Arrange
        String expectedToken = "admin-token";
        setupMockDsqlUtilities(expectedToken);

        try (MockedStatic<DsqlUtilities> dsqlUtilitiesMock = mockStatic(DsqlUtilities.class)) {
            dsqlUtilitiesMock.when(DsqlUtilities::builder).thenReturn(mockBuilder);

            // Act
            String token = TokenManager.getToken(TEST_HOSTNAME, TEST_REGION, ADMIN_USER, mockCredentialsProvider, TEST_DURATION);

            // Assert
            assertEquals(expectedToken, token);
            verify(mockDsqlUtilities).generateDbConnectAdminAuthToken(any(Consumer.class));
            verify(mockDsqlUtilities, never()).generateDbConnectAuthToken(any(Consumer.class));
        }
    }

    @Test
    void testGetToken_CacheHit_ReturnsCachedToken() throws SQLException {
        // Arrange
        String expectedToken = "cached-token";
        setupMockDsqlUtilities(expectedToken);

        try (MockedStatic<DsqlUtilities> dsqlUtilitiesMock = mockStatic(DsqlUtilities.class)) {
            dsqlUtilitiesMock.when(DsqlUtilities::builder).thenReturn(mockBuilder);

            // First call - generates token
            String firstToken = TokenManager.getToken(TEST_HOSTNAME, TEST_REGION, TEST_USER, mockCredentialsProvider, TEST_DURATION);
            
            // Second call - should return cached token
            String secondToken = TokenManager.getToken(TEST_HOSTNAME, TEST_REGION, TEST_USER, mockCredentialsProvider, TEST_DURATION);

            // Assert
            assertEquals(expectedToken, firstToken);
            assertEquals(expectedToken, secondToken);
            // Should only call token generation once
            verify(mockDsqlUtilities, times(1)).generateDbConnectAuthToken(any(Consumer.class));
        }
    }

    @Test
    void testGetToken_DifferentConfigurations_GeneratesSeparateTokens() throws SQLException {
        // Arrange
        String token1 = "token1";
        String token2 = "token2";
        AwsCredentialsProvider mockCredentialsProvider2 = mock(AwsCredentialsProvider.class);
        
        setupMockDsqlUtilities(token1);

        try (MockedStatic<DsqlUtilities> dsqlUtilitiesMock = mockStatic(DsqlUtilities.class)) {
            dsqlUtilitiesMock.when(DsqlUtilities::builder).thenReturn(mockBuilder);
            
            // Mock different responses for different calls
            when(mockDsqlUtilities.generateDbConnectAuthToken(any(Consumer.class)))
                .thenReturn(token1)
                .thenReturn(token2);

            // Act
            String firstToken = TokenManager.getToken(TEST_HOSTNAME, TEST_REGION, TEST_USER, mockCredentialsProvider, TEST_DURATION);
            String secondToken = TokenManager.getToken(TEST_HOSTNAME, TEST_REGION, "user2", mockCredentialsProvider2, TEST_DURATION);

            // Assert
            assertEquals(token1, firstToken);
            assertEquals(token2, secondToken);
            // Should call token generation twice for different configurations
            verify(mockDsqlUtilities, times(2)).generateDbConnectAuthToken(any(Consumer.class));
        }
    }

    @Test
    void testForceRefreshToken_AlwaysGeneratesNewToken() throws SQLException {
        // Arrange
        String initialToken = "initial-token";
        String refreshedToken = "refreshed-token";
        setupMockDsqlUtilities(initialToken);

        try (MockedStatic<DsqlUtilities> dsqlUtilitiesMock = mockStatic(DsqlUtilities.class)) {
            dsqlUtilitiesMock.when(DsqlUtilities::builder).thenReturn(mockBuilder);
            
            // Mock different responses for different calls
            when(mockDsqlUtilities.generateDbConnectAuthToken(any(Consumer.class)))
                .thenReturn(initialToken)
                .thenReturn(refreshedToken);

            // First call - generates initial token
            String firstToken = TokenManager.getToken(TEST_HOSTNAME, TEST_REGION, TEST_USER, mockCredentialsProvider, TEST_DURATION);
            
            // Force refresh - should generate new token
            String secondToken = TokenManager.forceRefreshToken(TEST_HOSTNAME, TEST_REGION, TEST_USER, mockCredentialsProvider, TEST_DURATION);

            // Assert
            assertEquals(initialToken, firstToken);
            assertEquals(refreshedToken, secondToken);
            // Should call token generation twice
            verify(mockDsqlUtilities, times(2)).generateDbConnectAuthToken(any(Consumer.class));
        }
    }

    @Test
    void testClearCache_RemovesAllCachedTokens() throws SQLException {
        // Arrange
        String expectedToken = "test-token";
        setupMockDsqlUtilities(expectedToken);

        try (MockedStatic<DsqlUtilities> dsqlUtilitiesMock = mockStatic(DsqlUtilities.class)) {
            dsqlUtilitiesMock.when(DsqlUtilities::builder).thenReturn(mockBuilder);

            // First call - generates and caches token
            TokenManager.getToken(TEST_HOSTNAME, TEST_REGION, TEST_USER, mockCredentialsProvider, TEST_DURATION);
            
            // Clear cache
            TokenManager.clearCache();
            
            // Second call - should generate new token since cache is cleared
            TokenManager.getToken(TEST_HOSTNAME, TEST_REGION, TEST_USER, mockCredentialsProvider, TEST_DURATION);

            // Assert
            // Should call token generation twice since cache was cleared
            verify(mockDsqlUtilities, times(2)).generateDbConnectAuthToken(any(Consumer.class));
        }
    }

    @Test
    void testGetToken_NullTokenDuration_UsesDefaultDuration() throws SQLException {
        // Arrange
        String expectedToken = "test-token";
        setupMockDsqlUtilities(expectedToken);

        try (MockedStatic<DsqlUtilities> dsqlUtilitiesMock = mockStatic(DsqlUtilities.class)) {
            dsqlUtilitiesMock.when(DsqlUtilities::builder).thenReturn(mockBuilder);

            // Act
            String token = TokenManager.getToken(TEST_HOSTNAME, TEST_REGION, TEST_USER, mockCredentialsProvider, null);

            // Assert
            assertEquals(expectedToken, token);
            verify(mockDsqlUtilities).generateDbConnectAuthToken(any(Consumer.class));
        }
    }

    @Test
    void testGetToken_TokenGenerationFails_ThrowsException() {
        // Arrange
        setupMockDsqlUtilitiesWithException();

        try (MockedStatic<DsqlUtilities> dsqlUtilitiesMock = mockStatic(DsqlUtilities.class)) {
            dsqlUtilitiesMock.when(DsqlUtilities::builder).thenReturn(mockBuilder);

            // Act & Assert
            SQLException exception = assertThrows(SQLException.class, () -> 
                TokenManager.getToken(TEST_HOSTNAME, TEST_REGION, TEST_USER, mockCredentialsProvider, TEST_DURATION));
            
            assertTrue(exception.getMessage().contains("Token generation failed"));
        }
    }

    private void setupMockDsqlUtilities(String token) {
        when(mockBuilder.region(any(Region.class))).thenReturn(mockBuilder);
        when(mockBuilder.credentialsProvider(any(AwsCredentialsProvider.class))).thenReturn(mockBuilder);
        when(mockBuilder.build()).thenReturn(mockDsqlUtilities);
        when(mockDsqlUtilities.generateDbConnectAuthToken(any(Consumer.class))).thenReturn(token);
        when(mockDsqlUtilities.generateDbConnectAdminAuthToken(any(Consumer.class))).thenReturn(token);
    }

    private void setupMockDsqlUtilitiesWithException() {
        when(mockBuilder.region(any(Region.class))).thenReturn(mockBuilder);
        when(mockBuilder.credentialsProvider(any(AwsCredentialsProvider.class))).thenReturn(mockBuilder);
        when(mockBuilder.build()).thenReturn(mockDsqlUtilities);
        when(mockDsqlUtilities.generateDbConnectAuthToken(any(Consumer.class)))
            .thenThrow(new RuntimeException("Token generation failed"));
    }
}
