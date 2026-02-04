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

import java.sql.Connection;
import java.sql.SQLException;
import java.util.Properties;
import org.postgresql.Driver;

class ConnWrapper {
    private static final String APPLICATION_NAME_KEY = "ApplicationName";
    private static final String BASE_APPLICATION_NAME = "aurora-dsql-jdbc/" + Version.FULL;
    // Setting assumeMinServerVersion to 9.0 or higher forces pgjdbc to send
    // application_name in the startup packet instead of via SET command.
    // This is required for DSQL relay logs to capture the ApplicationName.
    // See: https://github.com/pgjdbc/pgjdbc/issues/2114
    private static final String ASSUME_MIN_SERVER_VERSION_KEY = "assumeMinServerVersion";
    private static final String ASSUME_MIN_SERVER_VERSION_VALUE = "9.0";

    private final String url;
    private final Properties info;

    public ConnWrapper(final String url, final Properties info) {
        this.url = url;
        this.info = new Properties(info);
    }

    public Connection makeConnection() throws SQLException {
        // Set application_name with optional ORM prefix
        String applicationName = buildApplicationName(info.getProperty(APPLICATION_NAME_KEY));
        info.setProperty(APPLICATION_NAME_KEY, applicationName);

        // Set assumeMinServerVersion to force application_name into startup packet
        // This ensures DSQL relay logs capture the ApplicationName
        if (info.getProperty(ASSUME_MIN_SERVER_VERSION_KEY) == null) {
            info.setProperty(ASSUME_MIN_SERVER_VERSION_KEY, ASSUME_MIN_SERVER_VERSION_VALUE);
        }

        Driver driver = new Driver();
        return driver.connect(url, info);
    }

    /**
     * Build the application_name with optional ORM prefix.
     *
     * <p>If ormPrefix is provided and non-empty after trimming, prepends it to the connector
     * identifier. Otherwise, returns the connector's application_name.
     *
     * @param ormPrefix Optional ORM name to prepend (e.g., "hibernate")
     * @return Formatted application_name string
     */
    static String buildApplicationName(final String ormPrefix) {
        final String trimmed = ormPrefix != null ? ormPrefix.trim() : null;
        if (trimmed != null && !trimmed.isEmpty()) {
            return trimmed + ":" + BASE_APPLICATION_NAME;
        }
        return BASE_APPLICATION_NAME;
    }
}
