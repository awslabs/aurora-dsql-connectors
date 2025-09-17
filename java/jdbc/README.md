# Aurora DSQL JDBC Connector
[![Maven Central Version](https://img.shields.io/maven-central/v/software.amazon.dsql/aurora-dsql-jdbc-connector?style=for-the-badge)](https://central.sonatype.com/artifact/software.amazon.dsql/aurora-dsql-jdbc-connector)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg?style=for-the-badge)](LICENSE)

A PgJDBC connector that integrates IAM Authentication for connecting Java applications to Amazon Aurora DSQL clusters.

The **Aurora DSQL JDBC Connector** is designed as an authentication plugin that extends the functionality of the PostgreSQL JDBC driver to enable applications to authenticate with Amazon Aurora DSQL using IAM credentials. The connector does not connect directly to the database, but provides seamless IAM authentication on top of the underlying PostgreSQL JDBC driver.

The Aurora DSQL JDBC Connector is built to work with the [PostgreSQL JDBC Driver](https://github.com/pgjdbc/pgjdbc) and provides seamless integration with Amazon Aurora DSQL's IAM authentication requirements.

In conjunction with the PostgreSQL JDBC Driver, the Aurora DSQL JDBC Connector enables IAM-based authentication for Amazon Aurora DSQL. It introduces deep integration with AWS authentication services such as [AWS Identity and Access Management (IAM)](https://aws.amazon.com/iam/).

## About the Connector
Amazon Aurora DSQL is a distributed SQL database service that provides high availability and scalability for PostgreSQL-compatible applications. Aurora DSQL requires IAM-based authentication with time-limited tokens that existing JDBC drivers do not natively support.

The main idea behind the Aurora DSQL JDBC Connector is to add an authentication layer on top of the PostgreSQL JDBC driver that handles IAM token generation, allowing users to connect to Aurora DSQL without changing their existing JDBC workflows.

### What is Aurora DSQL Authentication?
In Amazon Aurora DSQL, **authentication** involves:
- **IAM Authentication**: All connections use IAM-based authentication with time-limited tokens
- **Token Generation**: Authentication tokens are generated using AWS credentials and have configurable lifetimes

The Aurora DSQL JDBC Connector is designed to understand these requirements and automatically generate IAM authentication tokens when establishing connections.

### Benefits of the Aurora DSQL JDBC Connector
Although Aurora DSQL provides a PostgreSQL-compatible interface, existing PostgreSQL drivers do not currently support Aurora DSQL's IAM authentication requirements. The Aurora DSQL JDBC Connector allows customers to continue using their existing PostgreSQL workflows while enabling IAM authentication through:

1. **Automatic Token Generation**: IAM tokens are generated automatically using AWS credentials
2. **Seamless Integration**: Works with existing JDBC connection patterns
3. **AWS Credentials Support**: Supports various AWS credential providers (default, profile-based, etc.)

### Using the Aurora DSQL JDBC Connector with Connection Pooling
The Aurora DSQL JDBC Connector works with connection pooling libraries such as HikariCP. The connector handles IAM token generation during connection establishment, allowing connection pools to operate normally.

## Getting Started
For more information on how to download the Aurora DSQL JDBC Connector, minimum requirements to use it, and how to integrate it within your project, please refer to the sections below.

### Prerequisites
- Java 8 or higher
- An existing Aurora DSQL cluster with `Active` status
- AWS credentials configured (via AWS CLI, environment variables, or IAM roles)

### Maven Central
The latest version is available on [Maven Central](https://central.sonatype.com/artifact/software.amazon.dsql/aurora-dsql-jdbc-connector). Check Maven Central or the badge at the top of the `README.md` for the latest version.

```xml
<dependency>
    <groupId>software.amazon.dsql</groupId>
    <artifactId>aurora-dsql-jdbc-connector</artifactId>
    <version><!-- See Maven Central for latest version --></version>
</dependency>
```

### Gradle Dependency
```kotlin
implementation("software.amazon.dsql:aurora-dsql-jdbc-connector:VERSION")
```

## Dependencies

The Aurora DSQL JDBC Connector automatically includes all necessary dependencies when added to your project:

- **PostgreSQL JDBC Driver**: Provides the underlying PostgreSQL connectivity that Aurora DSQL is compatible with
- **AWS SDK for Aurora DSQL**: Enables IAM authentication token generation for Aurora DSQL clusters

These dependencies are included transitively, so you don't need to add them explicitly to your build file. The specific versions and dependency tree can be found in the project's [build.gradle.kts](build.gradle.kts) file, or on [Maven Central](https://central.sonatype.com/artifact/software.amazon.dsql/aurora-dsql-jdbc-connector).

## Connection Methods

The Aurora DSQL JDBC Connector supports the following URL format for connecting to Aurora DSQL clusters:

### AWS DSQL PostgreSQL Connector Format

```java
// Using AWS DSQL PostgreSQL Connector prefix
String url = "jdbc:aws-dsql:postgresql://your-cluster.dsql.us-east-1.on.aws/postgres?user=admin";
Connection conn = DriverManager.getConnection(url);

// With additional parameters
String url = "jdbc:aws-dsql:postgresql://your-cluster.dsql.us-east-1.on.aws/postgres" +
            "?user=admin&token-duration-secs=14400&profile=myprofile";
Connection conn = DriverManager.getConnection(url);
```

**Benefits:**
- Clear identification of Aurora DSQL connections
- Works seamlessly with connection pooling libraries (HikariCP, etc.)
- Better integration with frameworks like Spring Boot
- Automatic IAM token generation and authentication

## Properties

| Parameter | Description                                                                                                               | Default                                                                                                              |
|-----------|---------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------|
| `user` | Determines the user for the connection and the token generation method used. Example: `admin`                             | -                                                                                                                    |
| `token-duration-secs` | Duration in seconds for token validity                                                                                    | [Same as dsql sdk limit](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/SECTION_authentication-token.html) |
| `profile` | Used for instantiating a `ProfileCredentialsProvider` for token generation with the provided profile name                 | -                                                                                                                    |
| `region` | AWS region for Aurora DSQL connections. It is optional. When provided, it will override the region extracted from the URL | -                                                                                                                    |
| `database` | The database name to connect to                                                                                           | postgres                                                                                                             |

## Logging
Enabling logging is a very useful mechanism for troubleshooting any issue one might potentially experience while using the Aurora DSQL JDBC Connector.

The connector uses Java's built-in logging system (java.util.logging). You can configure logging levels by creating a `logging.properties` file:

```properties
# Set root logger level to INFO for clean output
.level = INFO

# Show Aurora DSQL JDBC Connector FINE logs for detailed debugging
software.amazon.dsql.level = FINE

# Console handler configuration
handlers = java.util.logging.ConsoleHandler
java.util.logging.ConsoleHandler.level = FINE
java.util.logging.ConsoleHandler.formatter = java.util.logging.SimpleFormatter

# Detailed formatter pattern with timestamp and logger name
java.util.logging.SimpleFormatter.format = %1$tH:%1$tM:%1$tS.%1$tL [%4$s] %3$s - %5$s%n
```

## Examples

| Description | Examples                                                                                                                                                                                                               |
|-------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Using the Aurora DSQL JDBC Connector for basic connections with multiple configuration methods | [BasicConnectionExample](https://github.com/aws-samples/aurora-dsql-samples/tree/main/java/pgjdbc_using_dsql_connector/src/main/java/com/amazon/dsql/samples/BasicConnectionExample.java)                     |
| Using the Aurora DSQL JDBC Connector with custom AWS credentials providers | [CustomCredentialsProviderExample](https://github.com/aws-samples/aurora-dsql-samples/tree/main/java/pgjdbc_using_dsql_connector/src/main/java/com/amazon/dsql/samples/CustomCredentialsProviderExample.java) |
| Using HikariCP with the Aurora DSQL JDBC Connector for production-ready connection pooling | [HikariCP Example](https://github.com/aws-samples/aurora-dsql-samples/tree/main/java/pgjdbc_hikaricp_using_dsql_connector/src/main/java/org/example/Example.java)                                                      |
| Using Spring Boot with HikariCP and the Aurora DSQL JDBC Connector | [Spring Boot HikariCP Example](https://github.com/aws-samples/aurora-dsql-samples/tree/main/java/springboot_hikaricp_using_dsql_connector)                                                                             |


## Getting Help and Opening Issues
If you encounter a bug with the Aurora DSQL JDBC Connector, we would like to hear about it.
Please search the existing issues to see if others are also experiencing the issue before reporting the problem in a new issue.

When opening a new issue, please provide:
- Aurora DSQL cluster details (region, version)
- Java version and environment details
- Complete error messages and stack traces
- Steps to reproduce the issue
- Relevant configuration and code snippets

## How to Contribute
We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

1. Set up your environment by following the build instructions below
2. To contribute, first make a fork of this project
3. Make any changes on your fork. Make sure you are aware of the requirements for the project (e.g. Java 17+ support)
4. Create a pull request from your fork
5. Pull requests need to be approved and merged by maintainers into the main branch

**Note:** Before making a pull request, run all tests and verify everything is passing.

### Code Style
The project follows standard Java coding conventions. Please ensure your contributions maintain consistency with the existing codebase.

## Building the Project

This project uses Gradle 8.13 and follows the same build patterns as other AWS Aurora DSQL projects.

### Build Commands

```bash
# Build the project
./gradlew build

# Run tests
./gradlew test

# Generate JAR file
./gradlew jar

# Generate Javadoc
./gradlew javadoc

# Clean build artifacts
./gradlew clean
```

### Build Artifacts

The build generates the following artifacts:
- `aurora-dsql-jdbc-connector-<VERSION>.jar` - Main library JAR
- `aurora-dsql-jdbc-connector-<VERSION>-sources.jar` - Source code JAR
- `aurora-dsql-jdbc-connector-<VERSION>-javadoc.jar` - Javadoc JAR

### Code Quality

The project includes several code quality tools:

- **SpotBugs** - Static analysis for bug detection
- **JaCoCo** - Code coverage reporting

### Running Quality Checks

```bash
# Run all quality checks
./gradlew check

# Run only SpotBugs
./gradlew spotbugsMain spotbugsTest

# Generate code coverage report
./gradlew jacocoTestReport
```

## Aurora DSQL Version Testing
This `aurora-dsql-jdbc-connector` is being tested against Amazon Aurora DSQL clusters in our test suite.

The `aurora-dsql-jdbc-connector` is compatible with all Aurora DSQL cluster versions as it uses the standard PostgreSQL protocol for communication.

## License
This software is released under the Apache 2.0 license.

Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
SPDX-License-Identifier: Apache-2.0
