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

import java.lang.reflect.Modifier;
import java.util.Arrays;
import java.util.Collection;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;
import java.util.logging.Level;
import java.util.logging.Logger;

public final class PropertyDefinition {

    private PropertyDefinition() {
        throw new UnsupportedOperationException("This is a utility class and cannot be instantiated");
    }

    private static final Logger LOGGER = Logger.getLogger("com.amazon.jdbc.PropertyDefinition");

    public static final AuroraDsqlProperty USER = new AuroraDsqlProperty("user", null,
            "The user to connect with Aurora DSQL");

    public static final AuroraDsqlProperty PROFILE = new AuroraDsqlProperty("profile", null,
            "The profile to be used for Aurora DSQL connections");

    public static final AuroraDsqlProperty REGION = new AuroraDsqlProperty("region", null,
            "The AWS region for Aurora DSQL connections");

    // Token duration for long-lived tokens (default 8 hours = 28800 seconds)
    public static final AuroraDsqlProperty TOKEN_DURATION = new AuroraDsqlProperty("token-duration-secs", null,
            "The duration in seconds for cached tokens");
            
    // Database name for Aurora DSQL connections
    public static final AuroraDsqlProperty DATABASE = new AuroraDsqlProperty("database", "postgres",
            "The database name to connect to (default: postgres)");

    private static final Map<String, AuroraDsqlProperty> PROPS_BY_NAME =
            new ConcurrentHashMap<>();

    static {
        registerProperties();
    }

    public static Collection<AuroraDsqlProperty> getAllProperties() {
        return PROPS_BY_NAME.values();
    }

    private static void registerProperties() {
        Arrays.stream(PropertyDefinition.class.getDeclaredFields())
                .filter(
                        f ->
                                f.getType() == AuroraDsqlProperty.class
                                        && Modifier.isPublic(f.getModifiers())
                                        && Modifier.isStatic(f.getModifiers()))
                .forEach(
                        f -> {
                            AuroraDsqlProperty prop = null;
                            try {
                                prop = (AuroraDsqlProperty) f.get(AuroraDsqlProperty.class);
                            } catch (final IllegalArgumentException | IllegalAccessException ex) {
                                LOGGER.log(Level.WARNING, "An error has occurred while retrieving a property", ex);
                            }

                            if (prop != null) {
                                PROPS_BY_NAME.put(prop.name, prop);
                            }
                        });

    }
}
