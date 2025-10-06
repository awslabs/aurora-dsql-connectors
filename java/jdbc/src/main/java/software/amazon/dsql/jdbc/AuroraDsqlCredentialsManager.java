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

import software.amazon.awssdk.auth.credentials.AwsCredentialsProvider;
import software.amazon.awssdk.auth.credentials.DefaultCredentialsProvider;

/**
 * Manages the global AWS credentials provider used for IAM authentication token generation.
 *
 * <p>This utility class provides a centralized mechanism for configuring the {@link
 * AwsCredentialsProvider} used by {@link DSQLConnector} when establishing connections to Aurora
 * DSQL. By default, the connector uses the AWS SDK's {@link DefaultCredentialsProvider}, which
 * searches for credentials in standard locations. Applications can customize this behavior by
 * providing a custom credentials provider through {@link #setProvider(AwsCredentialsProvider)}.
 *
 * <h3>Default Behavior</h3>
 *
 * <p>When no custom provider is configured, the connector uses {@link DefaultCredentialsProvider},
 * which automatically searches for credentials in the AWS SDK's standard credential chain.
 *
 * <p>The {@link #resetProvider()} method can be used to restore the default behaviour.
 *
 * <h3>Custom Profile</h3>
 *
 * <p>For most use cases requiring a specific AWS profile, use the {@code profile} connection
 * property instead of configuring a custom provider through this class. The {@code profile}
 * property is simpler and takes precedence over the credentials provider configured here.
 *
 * <pre>{@code
 * Properties props = new Properties();
 * props.setProperty("user", "admin");
 * props.setProperty("profile", "production");
 * Connection conn = DriverManager.getConnection(url, props);
 * }</pre>
 *
 * <h3>Custom Credentials Provider</h3>
 *
 * <p>For advanced scenarios requiring credential sources beyond named profiles, configure a custom
 * provider:
 *
 * <pre>{@code
 * StsClient stsClient = StsClient.builder()
 *      .region(Region.US_EAST_1)
 *      .build();
 *
 * AwsCredentialsProvider assumeRoleCredentials = StsAssumeRoleCredentialsProvider.builder()
 *      .refreshRequest(
 *          AssumeRoleRequest.builder()
 *              .roleArn(ROLE_ARN)
 *              .roleSessionName("aurora-dsql-session")
 *              .durationSeconds(3600) // 1 hour
 *              .build()
 *      )
 *      .stsClient(stsClient)
 *      .build();
 *
 * AuroraDsqlCredentialsManager.setProvider(assumeRoleCredentials);
 *
 * Properties props = new Properties();
 * props.setProperty("user", "admin");
 * Connection conn = DriverManager.getConnection(url, props);
 * }</pre>
 *
 * <h3>Thread Safety</h3>
 *
 * <p>All methods in this class are thread-safe. The credentials provider can be safely configured
 * or retrieved from multiple threads concurrently.
 *
 * @see DSQLConnector#connect(String, java.util.Properties)
 * @see PropertyDefinition#PROFILE
 * @since 1.0.0
 */
public final class AuroraDsqlCredentialsManager {

    private AuroraDsqlCredentialsManager() {
        throw new UnsupportedOperationException(
                "This is a utility class and cannot be instantiated");
    }

    private static volatile AwsCredentialsProvider credentialsProvider =
            DefaultCredentialsProvider.builder().build();

    /**
     * Configures a custom AWS credentials provider for IAM token generation.
     *
     * <p>This method sets the global credentials provider used by all subsequent connections that
     * do not specify a {@code profile} property. The provider will be used to obtain AWS
     * credentials for generating IAM authentication tokens.
     *
     * <p><b>Note:</b> This setting is global and affects all connections created after this method
     * is called. Connections that specify a {@code profile} property will use that profile instead
     * of this provider.
     *
     * @param customCredentialsProvider the credentials provider to use for token generation
     * @throws NullPointerException if {@code customCredentialsProvider} is null
     * @see #resetProvider()
     * @see #getProvider()
     */
    public static void setProvider(final AwsCredentialsProvider customCredentialsProvider) {
        credentialsProvider = customCredentialsProvider;
    }

    /**
     * Resets the credentials provider to the default AWS SDK credential chain.
     *
     * <p>This method restores the credentials provider to {@link DefaultCredentialsProvider},
     * reverting any custom provider configured through {@link
     * #setProvider(AwsCredentialsProvider)}.
     *
     * @see #setProvider(AwsCredentialsProvider)
     * @see #getProvider()
     */
    public static void resetProvider() {
        credentialsProvider = DefaultCredentialsProvider.builder().build();
    }

    /**
     * Retrieves the currently configured AWS credentials provider.
     *
     * <p>Returns the credentials provider that will be used for IAM token generation when no {@code
     * profile} property is specified in the connection.
     *
     * @return the current credentials provider
     * @see #setProvider(AwsCredentialsProvider)
     * @see #resetProvider()
     */
    public static AwsCredentialsProvider getProvider() {
        return credentialsProvider;
    }
}
