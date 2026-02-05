// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

plugins {
    id("java")
    id("com.gradleup.shadow") version "9.2.2"
}

repositories {
    mavenCentral()
}

tasks.withType<JavaCompile> {
    options.release = project.property("targetJavaVersion").toString().toInt()
}

dependencies {
    testImplementation(project(":"))

    testImplementation("org.junit.jupiter:junit-jupiter:5.9.2")
    testRuntimeOnly("org.junit.platform:junit-platform-console-standalone:1.14.2")
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

    // Only run when feature flag property is set
    onlyIf {
        val shouldRun = System.getProperty("runIntegrationTests") == "true"
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

tasks.shadowJar {
    archiveClassifier.set("all")
    from(sourceSets.test.get().output)
    configurations = listOf(project.configurations.testRuntimeClasspath.get())
    manifest {
        attributes["Main-Class"] = "org.junit.platform.console.ConsoleLauncher"
    }
}

tasks.named("build") {
    dependsOn("shadowJar")
}
