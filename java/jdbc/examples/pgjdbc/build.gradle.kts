// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

import org.gradle.api.tasks.testing.logging.TestExceptionFormat

plugins {
    id("java")
    id("application")
}

application {
    mainClass = "software.amazon.dsql.examples.ExamplePreferred"
}

group = "software.amazon.dsql.examples"
version = "1.0-SNAPSHOT"

val connectorVersion: String = providers.environmentVariable("CONNECTOR_VERSION").getOrElse("1.4.0")

repositories {
    if (providers.environmentVariable("USE_MAVEN_LOCAL").isPresent) {
        mavenLocal()
    }
    mavenCentral()
}

dependencies {
    implementation("com.zaxxer:HikariCP:7.0.2")
    implementation("software.amazon.dsql:aurora-dsql-jdbc-connector:$connectorVersion")

    testImplementation(platform("org.junit:junit-bom:6.1.0"))
    testImplementation("org.junit.jupiter:junit-jupiter")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

tasks.test {
    useJUnitPlatform()

    testLogging {
        events("passed", "skipped", "failed", "standardOut", "standardError")
        exceptionFormat = TestExceptionFormat.FULL
    }
}

tasks.withType<Test> {
    this.testLogging {
        this.showStandardStreams = true
    }
}

tasks.withType<JavaExec> {
    this.enableAssertions = true
}
