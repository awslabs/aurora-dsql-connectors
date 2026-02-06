<a id="python/connector/v0.2.6"></a>
# [Aurora DSQL Connector for Python v0.2.6 (python/connector/v0.2.6)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/python/connector/v0.2.6) - 2026-02-06

This is a maintenance release with no user-facing code changes. It includes a fix to the CI/CD workflow configuration.

## What's Changed
* fix: correct PyPI URL in Python release workflow by [@amaksimo](https://github.com/amaksimo) in [#64](https://github.com/awslabs/aurora-dsql-connectors/pull/64)

**Full Changelog**: https://github.com/awslabs/aurora-dsql-connectors/compare/python/connector/v0.2.5...python/connector/v0.2.6

[Changes][python/connector/v0.2.6]


<a id="python/connector/v0.2.5"></a>
# [Aurora DSQL Connector for Python v0.2.5 (python/connector/v0.2.5)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/python/connector/v0.2.5) - 2026-02-06

This release adds a `RESET ALL` call to the asyncpg connection pool, which resets session state when connections are returned to the pool. This feature leverages the recently added `RESET ALL` support in Aurora DSQL.

This release also migrates the connector into the [aurora-dsql-connectors](https://github.com/awslabs/aurora-dsql-connectors) monorepo.

## What's Changed
* Adding a call to RESET ALL in asyncpg pool by [@leszek-bq](https://github.com/leszek-bq) in [#28](https://github.com/awslabs/aurora-dsql-connectors/pull/28)

**Full Changelog**: https://github.com/awslabs/aurora-dsql-connectors/compare/python/connector/v0.2.2...python/connector/v0.2.5

[Changes][python/connector/v0.2.5]


<a id="python/connector/v0.2.2"></a>
# [Aurora DSQL Connector for Python v0.2.2 (python/connector/v0.2.2)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/python/connector/v0.2.2) - 2026-02-04

> **Note:** This release was originally published on Dec 31, 2025 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-python-connector](https://github.com/awslabs/aurora-dsql-python-connector/releases/tag/0.2.2).

---

This release fixes an issue where the default system region was not used to expand a cluster ID to a full cluster endpoint, when the cluster ID was passed as the `host` kwarg. The release also improves error message clarity for missing parameters.

## What's Changed
* Use default region for host kwarg when not provided by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-python-connector#26](https://github.com/awslabs/aurora-dsql-python-connector/pull/26)
* Fix missing host error message in DSN parsing by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-python-connector#27](https://github.com/awslabs/aurora-dsql-python-connector/pull/27)
* Always run default region integration tests by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-python-connector#28](https://github.com/awslabs/aurora-dsql-python-connector/pull/28)
* Bump version from 0.2.1 to 0.2.2 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-python-connector#29](https://github.com/awslabs/aurora-dsql-python-connector/pull/29)


**Full Changelog**: https://github.com/awslabs/aurora-dsql-connectors/compare/python/connector/v0.2.1...python/connector/v0.2.2



[Changes][python/connector/v0.2.2]


<a id="python/connector/v0.2.1"></a>
# [Aurora DSQL Connector for Python v0.2.1 (python/connector/v0.2.1)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/python/connector/v0.2.1) - 2026-02-04

> **Note:** This release was originally published on Dec 29, 2025 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-python-connector](https://github.com/awslabs/aurora-dsql-python-connector/releases/tag/v0.2.1).

---

This release adds a parsing step which expands a cluster ID into the full cluster endpoint, when it is provided in the `host` kwarg. Previously, this functionality only worked when the cluster ID was provided as the DSN.

## What's Changed
* Fix a typo in readme by [@leszek-bq](https://github.com/leszek-bq) in [awslabs/aurora-dsql-python-connector#18](https://github.com/awslabs/aurora-dsql-python-connector/pull/18)
* Standardize folder structure and format by [@amaksimo](https://github.com/amaksimo) in [awslabs/aurora-dsql-python-connector#20](https://github.com/awslabs/aurora-dsql-python-connector/pull/20)
* Simplify example connection params by [@amaksimo](https://github.com/amaksimo) in [awslabs/aurora-dsql-python-connector#21](https://github.com/awslabs/aurora-dsql-python-connector/pull/21)
* Add example smoke tests to integration workflow by [@amaksimo](https://github.com/amaksimo) in [awslabs/aurora-dsql-python-connector#22](https://github.com/awslabs/aurora-dsql-python-connector/pull/22)
* Fix module paths for examples by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-python-connector#24](https://github.com/awslabs/aurora-dsql-python-connector/pull/24)
* Expand cluster ID to endpoint with host kwarg by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-python-connector#23](https://github.com/awslabs/aurora-dsql-python-connector/pull/23)
* Bump version from 0.2.0 to 0.2.1 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-python-connector#25](https://github.com/awslabs/aurora-dsql-python-connector/pull/25)

## New Contributors
* [@amaksimo](https://github.com/amaksimo) made their first contribution in [awslabs/aurora-dsql-python-connector#20](https://github.com/awslabs/aurora-dsql-python-connector/pull/20)
* [@danielfrankcom](https://github.com/danielfrankcom) made their first contribution in [awslabs/aurora-dsql-python-connector#24](https://github.com/awslabs/aurora-dsql-python-connector/pull/24)

**Full Changelog**: https://github.com/awslabs/aurora-dsql-connectors/compare/python/connector/v0.2.0...python/connector/v0.2.1



[Changes][python/connector/v0.2.1]


<a id="python/connector/v0.2.0"></a>
# [Aurora DSQL Connector for Python v0.2.0 (python/connector/v0.2.0)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/python/connector/v0.2.0) - 2026-02-04

> **Note:** This release was originally published on Nov 21, 2025 by [@leszek-bq](https://github.com/leszek-bq) in [awslabs/aurora-dsql-python-connector](https://github.com/awslabs/aurora-dsql-python-connector/releases/tag/v0.2.0).

---

Added support for the asyncpg client library.



[Changes][python/connector/v0.2.0]


<a id="python/connector/v0.1.1"></a>
# [Aurora DSQL Connector for Python v0.1.1 (python/connector/v0.1.1)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/python/connector/v0.1.1) - 2026-02-04

> **Note:** This release was originally published on Oct 31, 2025 by [@leszek-bq](https://github.com/leszek-bq) in [awslabs/aurora-dsql-python-connector](https://github.com/awslabs/aurora-dsql-python-connector/releases/tag/v0.1.1).

---

- Updated Links



[Changes][python/connector/v0.1.1]


<a id="python/connector/v0.1.0"></a>
# [Aurora DSQL Connector for Python v0.1.0 (python/connector/v0.1.0)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/python/connector/v0.1.0) - 2026-02-04

> **Note:** This release was originally published on Oct 31, 2025 by [@leszek-bq](https://github.com/leszek-bq) in [awslabs/aurora-dsql-python-connector](https://github.com/awslabs/aurora-dsql-python-connector/releases/tag/v0.1.0).

---

Initial Release



[Changes][python/connector/v0.1.0]


[python/connector/v0.2.6]: https://github.com/awslabs/aurora-dsql-connectors/compare/python/connector/v0.2.5...python/connector/v0.2.6
[python/connector/v0.2.5]: https://github.com/awslabs/aurora-dsql-connectors/compare/python/connector/v0.2.2...python/connector/v0.2.5
[python/connector/v0.2.2]: https://github.com/awslabs/aurora-dsql-connectors/compare/python/connector/v0.2.1...python/connector/v0.2.2
[python/connector/v0.2.1]: https://github.com/awslabs/aurora-dsql-connectors/compare/python/connector/v0.2.0...python/connector/v0.2.1
[python/connector/v0.2.0]: https://github.com/awslabs/aurora-dsql-connectors/compare/python/connector/v0.1.1...python/connector/v0.2.0
[python/connector/v0.1.1]: https://github.com/awslabs/aurora-dsql-connectors/compare/python/connector/v0.1.0...python/connector/v0.1.1
[python/connector/v0.1.0]: https://github.com/awslabs/aurora-dsql-connectors/tree/python/connector/v0.1.0

<!-- Generated by https://github.com/rhysd/changelog-from-release v3.9.1 -->
