<a id="node/node-postgres/v0.1.8"></a>
# [Aurora DSQL Connector for node-postgres v0.1.8 (node/node-postgres/v0.1.8)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/node/node-postgres/v0.1.8) - 2026-02-05

## What's Changed

### New Features
- Added `application_name` support for connection tracking and monitoring

### Full Changelog
https://github.com/awslabs/aurora-dsql-connectors/compare/node/node-postgres/v0.1.7...node/node-postgres/v0.1.8

[Changes][node/node-postgres/v0.1.8]


<a id="node/node-postgres/v0.1.7"></a>
# [Aurora DSQL Connector for node-postgres v0.1.7 (node/node-postgres/v0.1.7)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/node/node-postgres/v0.1.7) - 2026-02-04

> **Note:** This release was originally published on Dec 30, 2025 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector](https://github.com/awslabs/aurora-dsql-nodejs-connector/releases/tag/aurora-dsql-node-postgres-connector-v0.1.7).

---

This release fixes a bundling issue where `pg-connection-string` was being inlined into the built artifact, causing `import.meta.url` errors in certain bundler environments ([#158](https://github.com/awslabs/aurora-dsql-connectors/issues/158)). The library is now correctly treated as an external dependency, reducing ESM bundle size from 12.25kB to 6.20kB.

### Dependencies

To prevent further bundling issues, the `package.json` file was updated to include explicit dependencies for packages which were previously used transitively.

- Added `pg-connection-string` as explicit peer dependency. This is provided by `pg`, but we reference it directly.
- Added `@smithy/types` to dependencies. This is provided by the AWS SDK, but we reference it directly.

## What's Changed
* Standardize folder structure and format by [@amaksimo](https://github.com/amaksimo) in [awslabs/aurora-dsql-nodejs-connector#156](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/156)
* Update node-postgres example to use 0.1.6 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#155](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/155)
* Convert node-postgres examples from TypeScript to JavaScript by [@amaksimo](https://github.com/amaksimo) in [awslabs/aurora-dsql-nodejs-connector#157](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/157)
* Add explicit pg-connection-string dep to fix bundling issue by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#159](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/159)
* Add node-postgres dependency check for source code during CI by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#161](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/161)
* Add node-postgres publint check for built artifact by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#160](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/160)
* Add node-postgres attw check for built artifact by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#162](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/162)
* Add node-postgres eslint-plugin-import-x plugin check by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#167](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/167)
* Bump package versions by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#168](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/168)

## New Contributors
* [@amaksimo](https://github.com/amaksimo) made their first contribution in [awslabs/aurora-dsql-nodejs-connector#156](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/156)

**Full Changelog**: https://github.com/awslabs/aurora-dsql-connectors/compare/node/node-postgres/v0.1.6...node/node-postgres/v0.1.7



[Changes][node/node-postgres/v0.1.7]


<a id="node/node-postgres/v0.1.6"></a>
# [Aurora DSQL Connector for node-postgres v0.1.6 (node/node-postgres/v0.1.6)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/node/node-postgres/v0.1.6) - 2026-02-04

> **Note:** This release was originally published on Dec 22, 2025 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector](https://github.com/awslabs/aurora-dsql-nodejs-connector/releases/tag/aurora-dsql-node-postgres-connector-v0.1.6).

---

This release improves handling of `null` and `undefined` configuration values, when specified using either `AuroraDSQLConfig` or `AuroraDSQLPoolConfig`. `null` or `undefined` values will now be replaced with default values, which improves compatibility with other libraries.

## What's Changed
* Bump @aws/aurora-dsql-node-postgres-connector from 0.1.3 to 0.1.5 in /packages/node-postgres/example by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#140](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/140)
* Reduce Dependabot update spam by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#146](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/146)
* Don't override defaults with null/undefined config by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#145](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/145)
* Remove broken Dependabot daily updates by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#147](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/147)
* Bump tsdown from 0.17.3 to 0.18.0 in /packages/node-postgres by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#143](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/143)
* Bump @typescript-eslint/eslint-plugin from 8.49.0 to 8.50.0 in /packages/node-postgres by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#141](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/141)
* Bump @eslint/js from 9.39.1 to 9.39.2 in /packages/node-postgres by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#134](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/134)
* Bump the production group in /packages/node-postgres with 2 updates by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#148](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/148)
* Bump the development group across 1 directory with 5 updates by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#152](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/152)
* Bump node-postgres from 0.1.5 to 0.1.6 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#154](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/154)

**Full Changelog**: https://github.com/awslabs/aurora-dsql-connectors/compare/node/node-postgres/v0.1.5...node/node-postgres/v0.1.6



[Changes][node/node-postgres/v0.1.6]


<a id="node/node-postgres/v0.1.5"></a>
# [Aurora DSQL Connector for node-postgres v0.1.5 (node/node-postgres/v0.1.5)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/node/node-postgres/v0.1.5) - 2026-02-04

> **Note:** This release was originally published on Dec 12, 2025 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector](https://github.com/awslabs/aurora-dsql-nodejs-connector/releases/tag/aurora-dsql-node-postgres-connector-v0.1.5).

---

This release improves handling of the configured region, and parsing of the cluster ID. It also fixes a bug in the connection string logic merged in [#113](https://github.com/awslabs/aurora-dsql-connectors/issues/113).

## What's Changed
* Improve parsing of region and cluster ID by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#119](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/119)
* Bump @types/node from 25.0.0 to 25.0.1 in /packages/node-postgres by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#123](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/123)
* Bump tsdown from 0.17.2 to 0.17.3 in /packages/node-postgres by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#121](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/121)
* Prevent connection string parsing override with empty values by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#130](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/130)
* Add credentials provider integration tests by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#129](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/129)
* Bump node-postgres from 0.1.4 to 0.1.5 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#131](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/131)


**Full Changelog**: https://github.com/awslabs/aurora-dsql-connectors/compare/node/node-postgres/v0.1.4...node/node-postgres/v0.1.5



[Changes][node/node-postgres/v0.1.5]


<a id="node/node-postgres/v0.1.4"></a>
# [Aurora DSQL Connector for node-postgres v0.1.4 (node/node-postgres/v0.1.4)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/node/node-postgres/v0.1.4) - 2026-02-04

> **Note:** This release was originally published on Dec 12, 2025 by [@vic-tsang](https://github.com/vic-tsang) in [awslabs/aurora-dsql-nodejs-connector](https://github.com/awslabs/aurora-dsql-nodejs-connector/releases/tag/aurora-dsql-node-postgres-connector-v0.1.4).

---

This release fixes a bug where the `customCredentialsProvider` property was unused, despite being configured.

## What's Changed
* bugfix: Wire up custom credential providers to token generation by [@zakvdm](https://github.com/zakvdm) in [awslabs/aurora-dsql-nodejs-connector#120](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/120)
* update version to 0.1.4 by [@vic-tsang](https://github.com/vic-tsang) in [awslabs/aurora-dsql-nodejs-connector#128](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/128)

## New Contributors
* [@zakvdm](https://github.com/zakvdm) made their first contribution in [awslabs/aurora-dsql-nodejs-connector#120](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/120)

**Full Changelog**: https://github.com/awslabs/aurora-dsql-connectors/compare/node/node-postgres/v0.1.3...node/node-postgres/v0.1.4



[Changes][node/node-postgres/v0.1.4]


<a id="node/node-postgres/v0.1.3"></a>
# [Aurora DSQL Connector for node-postgres v0.1.3 (node/node-postgres/v0.1.3)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/node/node-postgres/v0.1.3) - 2026-02-04

> **Note:** This release was originally published on Dec 11, 2025 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector](https://github.com/awslabs/aurora-dsql-nodejs-connector/releases/tag/aurora-dsql-node-postgres-connector-v0.1.3).

---

This release adds a parsing step which processes the `connectionString` property from the `AuroraDSQLConfig` and `AuroraDSQLPoolConfig` objects.

## What's Changed
* parse from connectionString, check if region is set before parsing fr… by [@vic-tsang](https://github.com/vic-tsang) in [awslabs/aurora-dsql-nodejs-connector#113](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/113)
* Bump node-postgres from 0.1.2 to 0.1.3 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#117](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/117)


**Full Changelog**: https://github.com/awslabs/aurora-dsql-connectors/compare/node/node-postgres/v0.1.2...node/node-postgres/v0.1.3



[Changes][node/node-postgres/v0.1.3]


<a id="node/node-postgres/v0.1.2"></a>
# [Aurora DSQL Connector for node-postgres v0.1.2 (node/node-postgres/v0.1.2)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/node/node-postgres/v0.1.2) - 2026-02-04

> **Note:** This release was originally published on Dec 11, 2025 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector](https://github.com/awslabs/aurora-dsql-nodejs-connector/releases/tag/aurora-dsql-node-postgres-connector-v0.1.2).

---

  This release exports CommonJS files for consumption by CommonJS module users.

  This avoids import errors like the following, which occurred for non-ESM imports:

  ```
  src/index.ts:1:34 - error TS1479: The current file is a CommonJS module whose imports will produce 'require' calls;
however, the referenced file is an ECMAScript module and cannot be imported with 'require'. Consider writing a
dynamic 'import("@aws/aurora-dsql-node-postgres-connector")' call instead.

To convert this file to an ECMAScript module, change its file extension to '.mts' or create a local package.json
file with `{ "type": "module" }`.

  1 import { AuroraDSQLClient } from "@aws/aurora-dsql-node-postgres-connector";
                                     ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
  Found 1 error in src/index.ts:1
  ```

  ## What's Changed
  * Export node-postgres CommonJS module in addition to ES module by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#92](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/92)
  * Run node-postgres example smoke test as part of CI by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#91](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/91)
  * Add job to verify node-postgres versions by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#93](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/93)
  * Bump @types/node from 24.9.2 to 24.10.0 in /packages/node-postgres by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#73](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/73)
  * Configure Dependabot to monitor example projects by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#97](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/97)
  * Bump @types/pg from 8.15.6 to 8.16.0 in /packages/node-postgres by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#103](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/103)
  * Use non-deprecated testPathPatterns option for jest by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#107](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/107)
  * Bump jest from 29.7.0 to 30.2.0 in /packages/node-postgres/example by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#99](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/99)
  * Upgrade tsdown from 0.11.4 to 0.17.2 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#108](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/108)
  * Bump versions from 0.1.1 to 0.1.2 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#109](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/109)
  * Set up trusted publishing for node-postgres by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#114](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/114)
  * Fix node-postgres workflow permissions by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#115](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/115)

  ## New Contributors
  * [@danielfrankcom](https://github.com/danielfrankcom) made their first contribution in [awslabs/aurora-dsql-nodejs-connector#92](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/92)

  **Full Changelog**: https://github.com/awslabs/aurora-dsql-connectors/compare/node/node-postgres/v0.1.1...node/node-postgres/v0.1.2



[Changes][node/node-postgres/v0.1.2]


<a id="node/node-postgres/v0.1.1"></a>
# [Aurora DSQL Connector for node-postgres v0.1.1 (node/node-postgres/v0.1.1)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/node/node-postgres/v0.1.1) - 2026-02-04

> **Note:** This release was originally published on Nov 26, 2025 by [@mitchell-elholm](https://github.com/mitchell-elholm) in [awslabs/aurora-dsql-nodejs-connector](https://github.com/awslabs/aurora-dsql-nodejs-connector/releases/tag/aurora-dsql-node-postgres-connector-v0.1.1).

---

## What's Changed
- Bug fix for relative imports



[Changes][node/node-postgres/v0.1.1]


<a id="node/node-postgres/v0.1.0"></a>
# [Aurora DSQL Connector for node-postgres v0.1.0 (node/node-postgres/v0.1.0)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/node/node-postgres/v0.1.0) - 2026-02-04

> **Note:** This release was originally published on Oct 30, 2025 by [@mitchell-elholm](https://github.com/mitchell-elholm) in [awslabs/aurora-dsql-nodejs-connector](https://github.com/awslabs/aurora-dsql-nodejs-connector/releases/tag/aurora-dsql-node-postgres-connector-v0.1.0).

---

Initial release of the Aurora DSQL Connector for node-postgres



[Changes][node/node-postgres/v0.1.0]


[node/node-postgres/v0.1.8]: https://github.com/awslabs/aurora-dsql-connectors/compare/node/node-postgres/v0.1.7...node/node-postgres/v0.1.8
[node/node-postgres/v0.1.7]: https://github.com/awslabs/aurora-dsql-connectors/compare/node/node-postgres/v0.1.6...node/node-postgres/v0.1.7
[node/node-postgres/v0.1.6]: https://github.com/awslabs/aurora-dsql-connectors/compare/node/node-postgres/v0.1.5...node/node-postgres/v0.1.6
[node/node-postgres/v0.1.5]: https://github.com/awslabs/aurora-dsql-connectors/compare/node/node-postgres/v0.1.4...node/node-postgres/v0.1.5
[node/node-postgres/v0.1.4]: https://github.com/awslabs/aurora-dsql-connectors/compare/node/node-postgres/v0.1.3...node/node-postgres/v0.1.4
[node/node-postgres/v0.1.3]: https://github.com/awslabs/aurora-dsql-connectors/compare/node/node-postgres/v0.1.2...node/node-postgres/v0.1.3
[node/node-postgres/v0.1.2]: https://github.com/awslabs/aurora-dsql-connectors/compare/node/node-postgres/v0.1.1...node/node-postgres/v0.1.2
[node/node-postgres/v0.1.1]: https://github.com/awslabs/aurora-dsql-connectors/compare/node/node-postgres/v0.1.0...node/node-postgres/v0.1.1
[node/node-postgres/v0.1.0]: https://github.com/awslabs/aurora-dsql-connectors/tree/node/node-postgres/v0.1.0

<!-- Generated by https://github.com/rhysd/changelog-from-release v3.9.1 -->
