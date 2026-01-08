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

        Driver driver = new Driver();
        return driver.connect(url, info);
    }

    /**
     * Build the application_name with optional ORM prefix. If ormPrefix is provided (doesn't
     * contain '/'), prepend it to the connector name. Otherwise, use the connector's
     * application_name.
     */
    static String buildApplicationName(final String ormPrefix) {
        final String trimmed = ormPrefix != null ? ormPrefix.trim() : null;
        if (trimmed != null && !trimmed.isEmpty() && !trimmed.contains("/")) {
            return trimmed + ":" + BASE_APPLICATION_NAME;
        }
        return BASE_APPLICATION_NAME;
    }
}
