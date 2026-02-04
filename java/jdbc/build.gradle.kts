// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

import com.diffplug.spotless.FormatterFunc
import org.gradle.external.javadoc.StandardJavadocDocletOptions
import java.io.Serializable

// Temporary workaround for JReleaser/Spotless incompatibility.
// https://github.com/jreleaser/jreleaser/issues/1643
buildscript {
    configurations.classpath {
        resolutionStrategy {
            force("org.eclipse.jgit:org.eclipse.jgit:5.13.0.202109080827-r")
        }
    }
}

plugins {
    id("java-library")
    id("maven-publish")
    id("jacoco")
    id("com.diffplug.spotless") version "8.0.0"
    id("com.github.spotbugs") version "6.4.+"
    id("org.jreleaser") version "1.20.0"
}

group = "software.amazon.dsql"
version = System.getenv("PROJECT_VERSION") ?: "0.0.0-SNAPSHOT"

val targetJavaVersion = project.property("targetJavaVersion").toString().toInt()

repositories {
    mavenCentral()
}

val mockitoAgent = configurations.create("mockitoAgent")

dependencies {
    // AWS SDK for Aurora DSQL
    implementation("software.amazon.awssdk:dsql:2.41.21")

    // PostgreSQL JDBC Driver - core dependency for Aurora DSQL connector
    implementation("org.postgresql:postgresql:42.7.7")

    // Annotation dependencies for @Nullable, @Nonnull, etc.
    compileOnly("com.github.spotbugs:spotbugs-annotations:4.9.4")

    // Test dependencies
    testImplementation("org.junit.jupiter:junit-jupiter-api:5.9.2")
    testImplementation("org.junit.jupiter:junit-jupiter-engine:5.9.2")
    testImplementation("org.junit.jupiter:junit-jupiter:5.9.2")
    testImplementation("software.amazon.awssdk:regions:2.41.21")

    // Agent recommended for Java 21+ inline mocking.
    testImplementation("org.mockito:mockito-junit-jupiter:5.20.0")
    mockitoAgent("org.mockito:mockito-core:5.20.0") { isTransitive = false }

    // Runtime dependencies for tests
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

java {
    toolchain {
        languageVersion = JavaLanguageVersion.of(25)
    }
    withJavadocJar()
    withSourcesJar()
}

spotless {
    format("misc") {
        target("*.properties", "*.gradle", "*.md", ".gitignore")
        trimTrailingWhitespace()
        endWithNewline()
    }
    java {
        target("src/**/*.java", "integration-tests/src/**/*.java")
        licenseHeaderFile(".license-headers/java.txt")
        googleJavaFormat("1.29.0").aosp()
        removeUnusedImports()
        trimTrailingWhitespace()
        endWithNewline()
        custom(
            "Refuse wildcard imports",
            object : Serializable, FormatterFunc {
                override fun apply(input: String): String {
                    if (input.contains("\nimport .*\\*;".toRegex())) {
                        throw GradleException("No wildcard imports allowed.")
                    }
                    return input
                }
            },
        )
    }
    kotlinGradle {
        target("**/*.kts")
        licenseHeaderFile(".license-headers/kotlin.txt", "^(?!\\s*//)")
        ktlint()
        trimTrailingWhitespace()
        endWithNewline()
    }
}

tasks.named<Javadoc>("javadoc") {
    options.encoding = "UTF-8"
    (options as StandardJavadocDocletOptions).addStringOption("Xdoclint:none", "-quiet")
}

tasks.named<Jar>("jar") {
    manifest {
        attributes(
            "Implementation-Title" to "Aurora DSQL Connector for JDBC",
            "Implementation-Version" to version,
            "Implementation-Vendor" to "Amazon Web Services",
        )
    }
}

tasks.named<Test>("test") {
    useJUnitPlatform()
    jvmArgs("-javaagent:${mockitoAgent.asPath}")

    systemProperty("RUN_INTEGRATION", System.getenv("RUN_INTEGRATION") ?: "FALSE")

    // Pass environment variables to tests
    systemProperty("CLUSTER_ENDPOINT", System.getenv("CLUSTER_ENDPOINT") ?: "")
    systemProperty("CLUSTER_USER", System.getenv("CLUSTER_USER") ?: "")

    // Configure test execution
    testLogging {
        events("passed", "skipped", "failed", "standardOut", "standardError")
        exceptionFormat = org.gradle.api.tasks.testing.logging.TestExceptionFormat.FULL
    }

    // Set test timeout
    systemProperty("junit.jupiter.execution.timeout.default", "5m")
    systemProperty("junit.jupiter.execution.timeout.testable.method.default", "2m")
}

// Task specifically for integration tests
tasks.register<Test>("integrationTest") {
    description = "Runs integration tests against live Aurora DSQL cluster"
    group = "verification"

    System.setProperty("runIntegrationTests", "true")

    dependsOn(":integration-tests:test")
}

// Captures the version number in the jar so it can be provided on request.
tasks.register("generateVersionClass") {
    val versionStr = version.toString()
    val outputDir = layout.buildDirectory.dir("generated/sources/version/java")
    val versionFile = outputDir.map { it.file("software/amazon/dsql/jdbc/Version.java") }

    inputs.property("version", versionStr)
    outputs.file(versionFile)

    doLast {
        val versionRegex = """(\d+)\.(\d+)\.(\d+)(?:-.+)?""".toRegex()

        val matchResult =
            versionRegex.matchEntire(versionStr)
                ?: throw GradleException("Invalid version format: '$versionStr'")

        val (majorStr, minorStr, patchStr) = matchResult.destructured
        val major = majorStr.toInt()
        val minor = minorStr.toInt()
        val patch = patchStr.toInt()

        val file = versionFile.get().asFile
        file.parentFile.mkdirs()
        file.writeText(
            """
            package software.amazon.dsql.jdbc;

            /**
             * Version information for Aurora DSQL Connector for JDBC.
             * Generated automatically during build.
             *
             * @since 1.1.1
             */
            public final class Version {
                public static final int MAJOR = $major;
                public static final int MINOR = $minor;
                public static final int PATCH = $patch;
                public static final String FULL = "$versionStr";

                private Version() {
                    // Prevent construction of utility class.
                }
            }
            """.trimIndent(),
        )
    }
}

sourceSets {
    main {
        java {
            srcDir(layout.buildDirectory.dir("generated/sources/version/java"))
        }
    }
}

tasks.withType<JavaCompile> {
    dependsOn("generateVersionClass")
    options.encoding = "UTF-8"
    options.release = targetJavaVersion
    options.compilerArgs.addAll(listOf("-Werror", "-Xlint:deprecation", "-Xlint:-options"))
}

tasks.named("sourcesJar") {
    dependsOn("generateVersionClass")
}

publishing {
    publications {
        create<MavenPublication>("maven") {
            groupId = "software.amazon.dsql"
            artifactId = "aurora-dsql-jdbc-connector"

            from(components["java"])

            pom {
                name.set("Aurora DSQL Connector for JDBC")
                description.set("A pgJDBC connector that integrates IAM Authentication for Amazon Aurora DSQL clusters")
                url.set("https://github.com/awslabs/aurora-dsql-connectors/tree/main/java/jdbc")
                inceptionYear.set("2025")

                scm {
                    url.set("https://github.com/awslabs/aurora-dsql-connectors")
                    connection.set("scm:git:ssh://git@github.com/awslabs/aurora-dsql-connectors.git")
                    developerConnection.set("scm:git:ssh://git@github.com/awslabs/aurora-dsql-connectors.git")
                }

                licenses {
                    license {
                        name.set("The Apache License, Version 2.0")
                        url.set("https://www.apache.org/licenses/LICENSE-2.0.txt")
                        distribution.set("repo")
                    }
                }

                developers {
                    developer {
                        id.set("aws-aurora-dsql")
                        name.set("Aurora DSQL Team")
                        organization.set("Amazon Web Services")
                        organizationUrl.set("https://aws.amazon.com")
                    }
                }

                properties.set(
                    mapOf(
                        "maven.compiler.source" to targetJavaVersion.toString(),
                        "maven.compiler.target" to targetJavaVersion.toString(),
                    ),
                )
            }
        }
    }

    repositories {
        maven {
            url =
                layout.buildDirectory
                    .dir("staging-deploy")
                    .get()
                    .asFile
                    .toURI()
        }
    }
}

if ("UPLOAD".equals(System.getenv("JRELEASER_MAVENCENTRAL_STAGE"))) {
    jreleaser {
        project {
            name.set("aurora-dsql-jdbc-connector")
            description.set("A pgJDBC connector that integrates IAM Authentication for Amazon Aurora DSQL clusters")
            longDescription.set(
                "The Aurora DSQL Connector for JDBC is designed as an JDBC connector that extends the functionality of the PostgreSQL JDBC driver to enable applications to take full advantage of Amazon Aurora DSQL features.",
            )
            links {
                homepage.set("https://github.com/awslabs/aurora-dsql-connectors/tree/main/java/jdbc")
            }
            authors.set(listOf("Aurora DSQL Team"))
            license.set("Apache-2.0")
            copyright.set("2025 Amazon.com, Inc. or its affiliates. All Rights Reserved.")
        }

        signing {
            active.set(org.jreleaser.model.Active.ALWAYS)
            armored.set(true)
        }

        deploy {
            maven {
                mavenCentral {
                    register("sonatype") {
                        active.set(org.jreleaser.model.Active.ALWAYS)
                        url.set("https://central.sonatype.com/api/v1/publisher")
                        stagingRepository("build/staging-deploy")
                    }
                }
            }
        }
    }
}
