// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

plugins {
    id("java")
}

repositories {
    mavenLocal()
    mavenCentral()
}

dependencies {
    // Allow subproject to build standalone if the dependency is published in mavenLocal.
    if (System.getenv("USE_MAVEN_LOCAL") != null) {
        testImplementation("software.amazon.dsql:aurora-dsql-jdbc-connector:${System.getenv("PROJECT_VERSION")}")
    } else {
        testImplementation(project(":"))
    }

    testImplementation("org.junit.jupiter:junit-jupiter:5.9.2")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

sourceSets {
    test {
        java {
            srcDirs("src/test/java")
        }
    }
}

tasks.test {
    useJUnitPlatform()

    // Only run when feature flag property is set, or when run directly from this subproject
    onlyIf {
        val shouldRun =
            System.getProperty("runIntegrationTests") == "true" ||
                gradle.startParameter.currentDir.name == "integration-tests"
        if (!shouldRun) {
            println("Integration tests skipped - run task 'integrationTest' to run them")
        }
        shouldRun
    }

    // Always run integration tests, don't cache
    outputs.upToDateWhen { false }

    testLogging {
        events("passed", "skipped", "failed", "standardOut", "standardError")
        exceptionFormat = org.gradle.api.tasks.testing.logging.TestExceptionFormat.FULL
        showStandardStreams = true
    }

    systemProperty("junit.jupiter.execution.timeout.default", "10m")
    systemProperty("junit.jupiter.execution.timeout.testable.method.default", "5m")

    failFast = true
}
