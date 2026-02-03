# Contributing Guidelines

Thank you for your interest in contributing to the Aurora DSQL Connector for JDBC!

## How to contribute

### Reporting Bugs/Feature Requests

We welcome you to use the GitHub issue tracker to report bugs or suggest features.

When filing an issue, please check existing open, or recently closed, issues to make sure somebody else hasn't already
reported the issue. Please try to include as much information as you can. Details like these are incredibly useful:

* A reproducible test case or series of steps
* The version of our code being used
* Any modifications you've made relevant to the bug
* Anything unusual about your environment or deployment

### Contributing via Pull Requests

Contributions via pull requests are much appreciated. Before sending us a pull request, please ensure that:

1. You are working against the latest source on the *main* branch.
2. You check existing open, and recently merged, pull requests to make sure someone else hasn't addressed the problem already.
3. You open an issue to discuss any significant work - we would hate for your time to be wasted.

To send us a pull request, please:

1. Fork the repository.
2. Modify the source; please focus on the specific change you are contributing. If you also reformat all the code, it will be hard for us to focus on your change.
3. Ensure local tests pass.
4. Commit to your fork using clear commit messages.
5. Send us a pull request, answering any default questions in the pull request interface.
6. Pay attention to any automated CI failures reported in the pull request, and stay involved in the conversation.

GitHub provides additional document on [forking a repository](https://help.github.com/articles/fork-a-repo/) and
[creating a pull request](https://help.github.com/articles/creating-a-pull-request/).

## Development Environment Setup

### Prerequisites

* Java 25 or higher
* Git

### Building the Project

```bash
# Clone the repository
git clone https://github.com/awslabs/aurora-dsql-connectors.git
cd aurora-dsql-connectors/java/jdbc

# Build the project
./gradlew build

# Run tests
./gradlew test

# Generate code coverage report
./gradlew jacocoTestReport

# Run static analysis
./gradlew spotbugsMain spotbugsTest
```

### Running Tests

The project includes both unit tests and integration tests:

#### Unit Tests
```bash
# Run all unit tests
./gradlew test

# Run tests for a specific class
./gradlew test --tests "AuroraDsqlCredentialsManagerTest"

# Run tests with coverage
./gradlew test jacocoTestReport
```

#### Integration Tests
Integration tests require a real Aurora DSQL cluster and AWS credentials:

```bash
# Set required environment variables
export CLUSTER_ENDPOINT=your-cluster-endpoint.dsql.us-east-1.on.aws

# Optional: Set custom user (defaults to 'admin')
export CLUSTER_USER=your-username

# Run integration tests
./gradlew integrationTest
```

### Code Style and Quality

This project follows specific coding standards, which are defined by Spotless.

#### Quality Checks
```bash
# Run all quality checks
./gradlew check

# Run SpotBugs analysis
./gradlew spotbugsMain spotbugsTest

# Run Spotless checks
./gradlew spotlessCheck

# Apply Spotless fixes
./gradlew spotlessApply

# Generate code coverage report
./gradlew jacocoTestReport
```

#### Import Guidelines
**✅ Correct:**
```java
import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.SQLException;
```

**❌ Incorrect:**
```java
import java.sql.*;
```

### Testing Guidelines

#### Unit Test Structure
```java
@Test
void testMethodName_WhenCondition_ThenExpectedResult() {
    // Arrange
    String input = "test input";

    // Act
    String result = methodUnderTest(input);

    // Assert
    assertEquals("expected", result);
}
```

#### Integration Test Requirements
* Clean up resources in `@AfterEach` methods
* Use realistic test data
* Test with actual Aurora DSQL connections

#### Mock Usage
* Use Mockito for mocking dependencies
* Mock external dependencies, not the class under test
* Verify interactions when behavior matters
* Use `@SuppressFBWarnings` for test-specific SpotBugs warnings

### Documentation

#### Javadoc Requirements
All public APIs must include comprehensive Javadoc:

```java
/**
 * Creates a connection to Aurora DSQL with automatic token management.
 *
 * <p>This method handles:
 * <ul>
 *   <li>IAM token generation and refresh</li>
 *   <li>Connection lifetime management (60-minute limit)</li>
 *   <li>Prepared statement re-preparation on connection refresh</li>
 * </ul>
 *
 * @param url the Aurora DSQL cluster endpoint URL
 * @param properties connection properties including user and token settings
 * @return a connection connector with automatic management features
 * @throws SQLException if the connection cannot be established
 */
```

#### README Updates
Update the README.md file when:
* Adding new features or configuration options
* Changing existing behavior
* Adding new dependencies
* Modifying build or setup procedures

## Aurora DSQL Specific Considerations

When contributing to this connector, keep in mind Aurora DSQL's unique characteristics:

### Token Management
* IAM tokens have configurable lifetime
* Tokens are cached and refreshed automatically

## Commit Message Guidelines

Use clear, descriptive commit messages following conventional commit format:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
* `feat`: New feature
* `fix`: Bug fix
* `docs`: Documentation changes
* `test`: Test additions or modifications
* `refactor`: Code refactoring
* `perf`: Performance improvements
* `chore`: Maintenance tasks

Examples:
```
feat(connection): add automatic connection refresh
fix(token): resolve token expiration race condition
docs(readme): update configuration examples
test(integration): add long-lived connection tests
```

## Pull Request Process

1. **Create Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make Changes**
   * Follow coding standards
   * Add comprehensive tests
   * Update documentation
   * Ensure all checks pass

3. **Test Locally**
   ```bash
   ./gradlew build
   ./gradlew test
   ./gradlew check
   ```

4. **Commit Changes**
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   ```

5. **Push and Create PR**
   ```bash
   git push origin feature/your-feature-name
   ```

6. **PR Requirements**
   * Descriptive title and description
   * Reference related issues
   * Include test results
   * Update documentation as needed

## Issue Templates

### Bug Report
```markdown
**Describe the bug**
A clear and concise description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Create connection with properties '...'
2. Execute query '...'
3. See error

**Expected behavior**
A clear and concise description of what you expected to happen.

**Environment:**
- Java version: [e.g. 17]
- Aurora DSQL region: [e.g. us-east-1]
- Connector version: [e.g. 1.0.0]
- OS: [e.g. Ubuntu 20.04]

**Additional context**
Add any other context about the problem here, including stack traces.
```

### Feature Request
```markdown
**Is your feature request related to a problem? Please describe.**
A clear and concise description of what the problem is.

**Describe the solution you'd like**
A clear and concise description of what you want to happen.

**Describe alternatives you've considered**
A clear and concise description of any alternative solutions or features you've considered.

**Additional context**
Add any other context or screenshots about the feature request here.
```

## Security

If you discover a potential security issue in this project we ask that you notify AWS/Amazon Security via our [vulnerability reporting page](http://aws.amazon.com/security/vulnerability-reporting/). Please do **not** create a public GitHub issue.

## Licensing

See the [LICENSE](LICENSE) file for our project's licensing. We will ask you to confirm the licensing of your contribution.

We may ask you to sign a [Contributor License Agreement (CLA)](http://en.wikipedia.org/wiki/Contributor_License_Agreement) for larger changes.

## Code of Conduct

This project has adopted the [Amazon Open Source Code of Conduct](https://aws.github.io/code-of-conduct).
For more information see the [Code of Conduct FAQ](https://aws.github.io/code-of-conduct-faq) or contact
opensource-codeofconduct@amazon.com with any additional questions or comments.
