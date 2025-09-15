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

public final class AuroraDsqlCredentialsManager {

    private AuroraDsqlCredentialsManager() {
        throw new UnsupportedOperationException(
                "This is a utility class and cannot be instantiated");
    }

    private static volatile AwsCredentialsProvider credentialsProvider =
            DefaultCredentialsProvider.builder().build();

    /**
     * Set the AwsCredentialsProvider to use for token generation.
     *
     * @param customCredentialsProvider is an AwsCredentialsProvider (e.g.
     *     'ProfileCredentialProvider').
     */
    public static void setProvider(final AwsCredentialsProvider customCredentialsProvider) {
        credentialsProvider = customCredentialsProvider;
    }

    /** Resets AuroraDsqlCredentialsManager back to the DefaultCredentialsProvider. */
    public static void resetProvider() {
        credentialsProvider = DefaultCredentialsProvider.builder().build();
    }

    /** Retrieves AuroraDsqlCredentialsManager's credentials provider. */
    public static AwsCredentialsProvider getProvider() {
        return credentialsProvider;
    }
}
