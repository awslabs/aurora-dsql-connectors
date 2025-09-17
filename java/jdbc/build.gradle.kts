// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

import org.gradle.external.javadoc.StandardJavadocDocletOptions

plugins {
    id("java-library")
    id("maven-publish")
    id("jacoco")
    id("com.github.spotbugs") version "6.1.+"
    id("org.jreleaser") version "1.19.0"
}

group = "software.amazon.dsql"

val targetJavaVersion = project.property("targetJavaVersion").toString().toInt()

repositories {
    mavenCentral()
}

dependencies {
    // AWS SDK for Aurora DSQL
    implementation("software.amazon.awssdk:dsql:2.31.32")

    // PostgreSQL JDBC Driver - core dependency for Aurora DSQL connector
    implementation("org.postgresql:postgresql:42.7.7")

    // Annotation dependencies for @Nullable, @Nonnull, etc.
    implementation("javax.annotation:javax.annotation-api:1.3.2")
    implementation("com.github.spotbugs:spotbugs-annotations:4.7.3")

    // Test dependencies
    testImplementation("org.junit.jupiter:junit-jupiter-api:5.9.2")
    testImplementation("org.junit.jupiter:junit-jupiter-engine:5.9.2")
    testImplementation("software.amazon.awssdk:regions:2.31.32")
    testImplementation("software.amazon.awssdk:aws-core:2.31.32")
    testImplementation("org.junit.jupiter:junit-jupiter:5.9.2")
    testImplementation("org.mockito:mockito-junit-jupiter:5.1.1")

    // Runtime dependencies
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

java {
    toolchain {
        languageVersion = JavaLanguageVersion.of(17)
    }
    withJavadocJar()
    withSourcesJar()
}

tasks.named<Javadoc>("javadoc") {
    options.encoding = "UTF-8"
    (options as StandardJavadocDocletOptions).addStringOption("Xdoclint:none", "-quiet")
}

tasks.named<Jar>("jar") {
    manifest {
        attributes(
            "Implementation-Title" to "Aurora DSQL JDBC Connector",
            "Implementation-Version" to version,
            "Implementation-Vendor" to "Amazon Web Services"
        )
    }
}

tasks.named<Test>("test") {
    useJUnitPlatform()
    systemProperty("RUN_INTEGRATION", System.getenv("RUN_INTEGRATION") ?: "FALSE")

    // Pass environment variables to tests
    systemProperty("CLUSTER_ENDPOINT", System.getenv("CLUSTER_ENDPOINT") ?: "")
    systemProperty("REGION", System.getenv("REGION") ?: "")
    systemProperty("CLUSTER_USER", System.getenv("CLUSTER_USER") ?: "")

    // Configure test execution
    testLogging {
        events("passed", "skipped", "failed", "standardOut", "standardError")
        exceptionFormat = org.gradle.api.tasks.testing.logging.TestExceptionFormat.FULL
    }

    // Set test timeout
    systemProperty("junit.jupiter.execution.timeout.default", "5m")
    systemProperty("junit.jupiter.execution.timeout.testable.method.default", "2m")

    // Always run when invoked directly
    outputs.upToDateWhen {
        gradle.startParameter.taskNames.none { it == "test" || it == ":test" }
    }
}

// Task specifically for integration tests
tasks.register<Test>("integrationTest") {
    description = "Runs integration tests against live Aurora DSQL cluster"
    group = "verification"

    System.setProperty("runIntegrationTests", "true")

    dependsOn(":integration-tests:test")
}

tasks.withType<JavaCompile> {
    options.encoding = "UTF-8"
    options.release = targetJavaVersion
}

// SpotBugs configuration temporarily disabled for Brazil build compatibility
// TODO: Re-enable SpotBugs configuration once Brazil build environment supports external plugin repositories

publishing {
    publications {
        create<MavenPublication>("maven") {
            groupId = "software.amazon.dsql"
            artifactId = "aurora-dsql-jdbc-connector"

            from(components["java"])

            pom {
                name.set("Aurora DSQL JDBC Connector")
                description.set("A PgJDBC connector that integrates IAM Authentication for Amazon Aurora DSQL clusters")
                url.set("https://github.com/awslabs/aurora-dsql-jdbc-connector")
                inceptionYear.set("2025")

                scm {
                    url.set("https://github.com/awslabs/aurora-dsql-jdbc-connector")
                    connection.set("scm:git:ssh://git@github.com/awslabs/aurora-dsql-jdbc-connector.git")
                    developerConnection.set("scm:git:ssh://git@github.com/awslabs/aurora-dsql-jdbc-connector.git")
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

                properties.set(mapOf(
                    "maven.compiler.source" to targetJavaVersion.toString(),
                    "maven.compiler.target" to targetJavaVersion.toString()
                ))
            }
        }
    }

    repositories {
        maven {
            url = layout.buildDirectory.dir("staging-deploy").get().asFile.toURI()
        }
    }
}

if ("UPLOAD".equals(System.getenv("JRELEASER_MAVENCENTRAL_STAGE"))) {
    jreleaser {
        project {
            name.set("aurora-dsql-jdbc-connector")
            description.set("A PgJDBC connector that integrates IAM Authentication for Amazon Aurora DSQL clusters")
            longDescription.set("The Aurora DSQL JDBC Connector is designed as an JDBC connector that extends the functionality of the PostgreSQL JDBC driver to enable applications to take full advantage of Amazon Aurora DSQL features.")
            links {
                homepage.set("https://github.com/awslabs/aurora-dsql-jdbc-connector")
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
                    create("sonatype") {
                        active.set(org.jreleaser.model.Active.ALWAYS)
                        url.set("https://central.sonatype.com/api/v1/publisher")
                        stagingRepository("build/staging-deploy")
                    }
                }
            }
        }
    }
}
