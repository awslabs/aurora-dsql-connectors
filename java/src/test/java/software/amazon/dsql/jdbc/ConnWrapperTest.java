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
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

class ConnWrapperTest {

    @Test
    void testBuildApplicationName_Default() {
        String result = ConnWrapper.buildApplicationName(null);

        assertTrue(result.startsWith("aurora-dsql-jdbc/"));
        assertTrue(result.matches("aurora-dsql-jdbc/\\d+\\.\\d+\\.\\d+.*"));
    }

    @Test
    void testBuildApplicationName_WithOrmPrefix() {
        String result = ConnWrapper.buildApplicationName("hibernate");

        assertTrue(result.startsWith("hibernate:aurora-dsql-jdbc/"));
    }

    @Test
    void testBuildApplicationName_EmptyPrefix() {
        String result = ConnWrapper.buildApplicationName("");

        assertTrue(result.startsWith("aurora-dsql-jdbc/"));
    }

    @Test
    void testBuildApplicationName_WhitespacePrefix() {
        String result = ConnWrapper.buildApplicationName("   ");

        assertTrue(result.startsWith("aurora-dsql-jdbc/"));
        assertTrue(!result.contains(":"));
    }

    @Test
    void testBuildApplicationName_VeryLongPrefix() {
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < 100; i++) {
            sb.append("a");
        }
        String longPrefix = sb.toString();
        String result = ConnWrapper.buildApplicationName(longPrefix);

        assertTrue(result.startsWith(longPrefix + ":"));
        // We don't truncate - let PostgreSQL handle it
    }

    @Test
    void testBuildApplicationName_WithSpecialCharacters() {
        String result = ConnWrapper.buildApplicationName("my-app@2.0");

        assertTrue(result.startsWith("my-app@2.0:aurora-dsql-jdbc/"));
    }

    @Test
    void testBuildApplicationName_ContainsVersion() {
        String result = ConnWrapper.buildApplicationName(null);

        assertEquals("aurora-dsql-jdbc/" + Version.FULL, result);
    }
}
