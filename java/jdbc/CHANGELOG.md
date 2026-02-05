<a id="java/jdbc/v1.3.0"></a>
# [Aurora DSQL Connector for JDBC v1.3.0 (java/jdbc/v1.3.0)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/java/jdbc/v1.3.0) - 2026-02-04

> **Note:** This release was originally published on Oct 15, 2025 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector](https://github.com/awslabs/aurora-dsql-jdbc-connector/releases/tag/1.3.0).

---

This release clarifies the public API, and validates support for JDK 25.

Key changes:
- Release [1.0.0](https://github.com/awslabs/aurora-dsql-jdbc-connector/releases/tag/1.0.0) inadvertently specified internal implementation detail classes as `public`. This has been resolved in this release, with these classes now defined as `package-private`. Code that was depending on these implementation details should migrate to use the public API to access the functionality of this library. See the [published Javadocs](https://javadoc.io/doc/software.amazon.dsql/aurora-dsql-jdbc-connector/1.3.0/software/amazon/dsql/jdbc/package-summary.html).
- Support for JDK 25 is now validated as part of CI. It is likely that previous versions of the library were equally compatible with JDK 25, but this compatibility is now explicitly tested and verified.

## What's Changed
* Run integration tests from prebuilt jar file by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#34](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/34)
* Make internal detail classes package-private by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#31](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/31)
* Upgrade Mockito to silence warnings by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#32](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/32)
* Upgrade Gradle 8.14 -> 9.1.0 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#33](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/33)
* Update JDK 24 -> 25 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#36](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/36)
* Enable Gradle configuration cache by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#35](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/35)
* Add workarounds for JReleaser incompatibility issues by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#37](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/37)


**Full Changelog**: https://github.com/awslabs/aurora-dsql-connectors/compare/java/jdbc/v1.2.0...java/jdbc/v1.3.0



[Changes][java/jdbc/v1.3.0]


<a id="java/jdbc/v1.2.0"></a>
# [Aurora DSQL Connector for JDBC v1.2.0 (java/jdbc/v1.2.0)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/java/jdbc/v1.2.0) - 2026-02-04

> **Note:** This release was originally published on Oct 06, 2025 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector](https://github.com/awslabs/aurora-dsql-jdbc-connector/releases/tag/1.2.0).

---

This release includes improvements to logging functionality and fixes for database property parsing.

Key changes:

- Fixed database property parsing in connection URLs to properly handle the database parameter with consistent semantics across all properties
- Corrected logger package names to use the published package name (`software.amazon.dsql.jdbc`) consistently throughout the project. If you were previously configuring log levels using `com.amazon.jdbc`, you can now remove that configuration as `software.amazon.dsql.jdbc` is the only package name needed.
- Removed the `LazyLogger` class in favor of the standard `java.util.logging.Logger` which already provides equivalent functionality. If you were using `LazyLogger` directly, please use `java.util.logging.Logger` instead, which already includes built-in lazy evaluation for disabled log levels.

## What's Changed
* Remove LazyLogger class by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#28](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/28)
* Fix database property parsing by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#27](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/27)
* Fix logger package names by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#29](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/29)
* Avoid unnecessary operations for disabled log levels by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#30](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/30)

**Full Changelog**: https://github.com/awslabs/aurora-dsql-connectors/compare/java/jdbc/v1.1.2...java/jdbc/v1.2.0



[Changes][java/jdbc/v1.2.0]


<a id="java/jdbc/v1.1.2"></a>
# [Aurora DSQL Connector for JDBC v1.1.2 (java/jdbc/v1.1.2)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/java/jdbc/v1.1.2) - 2026-02-04

> **Note:** This release was originally published on Oct 06, 2025 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector](https://github.com/awslabs/aurora-dsql-jdbc-connector/releases/tag/1.1.2).

---

This release improves the Javadocs, aiming to provide a complete walkthrough for users.

## What's Changed
* Add javadoc badge by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#24](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/24)
* Minimize workflow permissions by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#25](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/25)
* Provide complete public API documentation by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#26](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/26)


**Full Changelog**: https://github.com/awslabs/aurora-dsql-connectors/compare/java/jdbc/v1.1.1...java/jdbc/v1.1.2



[Changes][java/jdbc/v1.1.2]


<a id="java/jdbc/v1.1.1"></a>
# [Aurora DSQL Connector for JDBC v1.1.1 (java/jdbc/v1.1.1)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/java/jdbc/v1.1.1) - 2026-02-04

> **Note:** This release was originally published on Oct 01, 2025 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector](https://github.com/awslabs/aurora-dsql-jdbc-connector/releases/tag/1.1.1).

---

This release fixes the version number reported by the library.

## What's Changed
* Embed dynamic version number in jar file by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#23](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/23)


**Full Changelog**: https://github.com/awslabs/aurora-dsql-connectors/compare/java/jdbc/v1.1.0...java/jdbc/v1.1.1



[Changes][java/jdbc/v1.1.1]


<a id="java/jdbc/v1.1.0"></a>
# [Aurora DSQL Connector for JDBC v1.1.0 (java/jdbc/v1.1.0)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/java/jdbc/v1.1.0) - 2026-02-04

> **Note:** This release was originally published on Sep 17, 2025 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector](https://github.com/awslabs/aurora-dsql-jdbc-connector/releases/tag/1.1.0).

---

## What's Changed

- Connector now targets compatibility with Java 8+
- Runtime dependency on `javax.annotation-api` and `spotbugs-annotations` removed

## Details

* Add Maven Central release badge by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#9](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/9)
* Move integration tests to standalone subproject by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#11](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/11)
* Target Java 8 as compatible version by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#12](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/12)
* Update JDK 17 -> 24 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#13](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/13)
* Update Gradle dependencies by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#14](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/14)
* Configure linting with Spotless by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#15](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/15)
* Fix Git blame ignore rev after merge by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#19](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/19)
* Avoid explicit version reference in source by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#16](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/16)
* Align code and docs by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#17](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/17)
* Remove runtime spotbugs dependency by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#18](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/18)
* Remove version number references from README by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#20](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/20)
* Fix publish workflow JDK version by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#21](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/21)
* Use explicit mavenLocal env var for standalone integration tests by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-jdbc-connector#22](https://github.com/awslabs/aurora-dsql-jdbc-connector/pull/22)


**Full Changelog**: https://github.com/awslabs/aurora-dsql-connectors/compare/java/jdbc/v1.0.0...java/jdbc/v1.1.0



[Changes][java/jdbc/v1.1.0]


<a id="java/jdbc/v1.0.0"></a>
# [Aurora DSQL Connector for JDBC v1.0.0 (java/jdbc/v1.0.0)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/java/jdbc/v1.0.0) - 2026-02-04

> **Note:** This release was originally published on Aug 26, 2025 by [@deepakscomk](https://github.com/deepakscomk) in [awslabs/aurora-dsql-jdbc-connector](https://github.com/awslabs/aurora-dsql-jdbc-connector/releases/tag/1.0.0).

---

Initial release of Aurora DSQL JDBC Connector



[Changes][java/jdbc/v1.0.0]


[java/jdbc/v1.3.0]: https://github.com/awslabs/aurora-dsql-connectors/compare/java/jdbc/v1.2.0...java/jdbc/v1.3.0
[java/jdbc/v1.2.0]: https://github.com/awslabs/aurora-dsql-connectors/compare/java/jdbc/v1.1.2...java/jdbc/v1.2.0
[java/jdbc/v1.1.2]: https://github.com/awslabs/aurora-dsql-connectors/compare/java/jdbc/v1.1.1...java/jdbc/v1.1.2
[java/jdbc/v1.1.1]: https://github.com/awslabs/aurora-dsql-connectors/compare/java/jdbc/v1.1.0...java/jdbc/v1.1.1
[java/jdbc/v1.1.0]: https://github.com/awslabs/aurora-dsql-connectors/compare/java/jdbc/v1.0.0...java/jdbc/v1.1.0
[java/jdbc/v1.0.0]: https://github.com/awslabs/aurora-dsql-connectors/tree/java/jdbc/v1.0.0

<!-- Generated by https://github.com/rhysd/changelog-from-release v3.9.1 -->
