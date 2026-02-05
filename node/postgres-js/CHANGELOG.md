<a id="node/postgres-js/v0.2.1"></a>
# [Aurora DSQL Connector for Postgres.js v0.2.1 (node/postgres-js/v0.2.1)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/node/postgres-js/v0.2.1) - 2026-02-05

## What's Changed

### New Features
- Added `application_name` support for connection tracking and monitoring

### Full Changelog
https://github.com/awslabs/aurora-dsql-connectors/compare/node/postgres-js/v0.2.0...node/postgres-js/v0.2.1

[Changes][node/postgres-js/v0.2.1]


<a id="node/postgres-js/v0.2.0"></a>
# [Aurora DSQL Connector for Postgres.js v0.2.0 (node/postgres-js/v0.2.0)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/node/postgres-js/v0.2.0) - 2026-02-04

> **Note:** This release was originally published on Jan 17, 2026 by [@vic-tsang](https://github.com/vic-tsang) in [awslabs/aurora-dsql-nodejs-connector](https://github.com/awslabs/aurora-dsql-nodejs-connector/releases/tag/aurora-dsql-postgresjs-connector-v0.2.0).

---

This release brings WebSocket support for Aurora DSQL connectivity. 

## What's Changed
* Update example dependency versions by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#169](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/169)
* Bump the production group in /packages/postgres-js with 3 updates by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#171](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/171)
* Bump the development group in /packages/postgres-js with 3 updates by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#173](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/173)
* Add application_name for usage metrics tracking by [@amaksimo](https://github.com/amaksimo) in [awslabs/aurora-dsql-nodejs-connector#175](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/175)
* Simplify application_name handling by [@amaksimo](https://github.com/amaksimo) in [awslabs/aurora-dsql-nodejs-connector#176](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/176)
* Unmodified postgres.js community test by [@vic-tsang](https://github.com/vic-tsang) in [awslabs/aurora-dsql-nodejs-connector#178](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/178)
* Add support for websocket by [@vic-tsang](https://github.com/vic-tsang) in [awslabs/aurora-dsql-nodejs-connector#86](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/86)
* Updated readme for new web socket feature by [@vic-tsang](https://github.com/vic-tsang) in [awslabs/aurora-dsql-nodejs-connector#179](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/179)
* Bump js-yaml from 4.1.0 to 4.1.1 in /packages/postgres-js by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#180](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/180)
* Updated to version 0.2.0 by [@vic-tsang](https://github.com/vic-tsang) in [awslabs/aurora-dsql-nodejs-connector#181](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/181)
* Update README.md by [@vic-tsang](https://github.com/vic-tsang) in [awslabs/aurora-dsql-nodejs-connector#183](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/183)
* Fix community test - clean up tables before each run by [@vic-tsang](https://github.com/vic-tsang) in [awslabs/aurora-dsql-nodejs-connector#182](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/182)


**Full Changelog**: https://github.com/awslabs/aurora-dsql-connectors/compare/node/postgres-js/v0.1.3...node/postgres-js/v0.2.0



[Changes][node/postgres-js/v0.2.0]


<a id="node/postgres-js/v0.1.3"></a>
# [Aurora DSQL Connector for Postgres.js v0.1.3 (node/postgres-js/v0.1.3)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/node/postgres-js/v0.1.3) - 2026-02-04

> **Note:** This release was originally published on Dec 30, 2025 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector](https://github.com/awslabs/aurora-dsql-nodejs-connector/releases/tag/aurora-dsql-postgresjs-connector-v0.1.3).

---

To prevent bundling issues, the `package.json` file was updated to include explicit dependencies for packages which were previously used transitively.

- Added @aws-sdk/types to dependencies. This is provided by the AWS SDK, but we reference it directly.

## What's Changed
* Bump @aws/aurora-dsql-postgresjs-connector from 0.1.1 to 0.1.2 in /packages/postgres-js/example by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#125](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/125)
* Bump @types/node from 25.0.0 to 25.0.1 in /packages/postgres-js by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#124](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/124)
* Bump tsdown from 0.17.2 to 0.17.3 in /packages/postgres-js by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#122](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/122)
* Add credentials provider integration tests by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#129](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/129)
* Reduce Dependabot update spam by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#146](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/146)
* Remove broken Dependabot daily updates by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#147](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/147)
* Bump tsdown from 0.17.3 to 0.18.0 in /packages/postgres-js by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#144](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/144)
* Bump @typescript-eslint/eslint-plugin from 8.49.0 to 8.50.0 in /packages/postgres-js by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#142](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/142)
* Bump eslint from 9.39.1 to 9.39.2 in /packages/postgres-js by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#138](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/138)
* Bump @eslint/js from 9.39.1 to 9.39.2 in /packages/postgres-js by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#137](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/137)
* Bump @types/node from 25.0.1 to 25.0.2 in /packages/postgres-js by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#136](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/136)
* Bump the production group in /packages/postgres-js with 2 updates by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#149](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/149)
* Bump the development group across 1 directory with 5 updates by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#153](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/153)
* Standardize folder structure and format by [@amaksimo](https://github.com/amaksimo) in [awslabs/aurora-dsql-nodejs-connector#156](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/156)
* Add postgres.js publint check for built artifact by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#163](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/163)
* Add postgres.js attw check for built artifact by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#164](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/164)
* Add postgres.js dependency check for source code during CI by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#165](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/165)
* Add postgres.js eslint-plugin-import-x plugin check by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#166](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/166)
* Bump package versions by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#168](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/168)

## New Contributors
* [@zakvdm](https://github.com/zakvdm) made their first contribution in [awslabs/aurora-dsql-nodejs-connector#120](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/120)
* [@amaksimo](https://github.com/amaksimo) made their first contribution in [awslabs/aurora-dsql-nodejs-connector#156](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/156)

**Full Changelog**: https://github.com/awslabs/aurora-dsql-connectors/compare/node/postgres-js/v0.1.2...node/postgres-js/v0.1.3



[Changes][node/postgres-js/v0.1.3]


<a id="node/postgres-js/v0.1.2"></a>
# [Aurora DSQL Connector for Postgres.js v0.1.2 (node/postgres-js/v0.1.2)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/node/postgres-js/v0.1.2) - 2026-02-04

> **Note:** This release was originally published on Dec 11, 2025 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector](https://github.com/awslabs/aurora-dsql-nodejs-connector/releases/tag/aurora-dsql-postgresjs-connector-v0.1.2).

---

This release exports CommonJS files for consumption by CommonJS module users.

This avoids import errors like the following, which occurred for non-ESM imports:

```
src/index.ts(1,36): error TS1479: The current file is a CommonJS module whose imports will produce 'require' calls;
however, the referenced file is an ECMAScript module and cannot be imported with 'require'. Consider writing a
dynamic 'import("@aws/aurora-dsql-postgresjs-connector")' call instead.

To convert this file to an ECMAScript module, change its file extension to '.mts' or create a local package.json
file with `{ "type": "module" }`.

1 import { auroraDSQLPostgres } from "@aws/aurora-dsql-postgresjs-connector";
                                     ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Found 1 error in src/index.ts:1
```

## What's Changed
* Export postgres.js CommonJS module in addition to ES module by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#94](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/94)
* Run postgres.js example smoke test as part of CI by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#95](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/95)
* Add job to verify postgres.js versions by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#96](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/96)
* Bump @aws-sdk/dsql-signer from 3.920.0 to 3.922.0 in /packages/postgres-js by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#77](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/77)
* Bump @types/node from 20.19.24 to 24.10.0 in /packages/postgres-js by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#78](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/78)
* Configure Dependabot to monitor example projects by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#97](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/97)
* Bump @typescript-eslint/parser from 8.48.0 to 8.49.0 in /packages/postgres-js by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#98](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/98)
* Bump ts-jest from 29.4.5 to 29.4.6 in /packages/postgres-js by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#100](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/100)
* Bump @typescript-eslint/eslint-plugin from 8.48.0 to 8.49.0 in /packages/postgres-js by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#102](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/102)
* Bump uuid from 11.1.0 to 13.0.0 in /packages/postgres-js/example by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#105](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/105)
* Use non-deprecated testPathPatterns option for jest by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#107](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/107)
* Bump jest from 29.7.0 to 30.2.0 in /packages/postgres-js/example by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#104](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/104)
* Upgrade tsdown from 0.11.4 to 0.17.2 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#108](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/108)
* Bump @typescript-eslint/eslint-plugin from 8.48.0 to 8.49.0 in /packages/postgres-js by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#112](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/112)
* Bump @typescript-eslint/parser from 8.48.0 to 8.49.0 in /packages/postgres-js by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#110](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/110)
* Bump ts-jest from 29.4.5 to 29.4.6 in /packages/postgres-js by [@dependabot](https://github.com/dependabot)[bot] in [awslabs/aurora-dsql-nodejs-connector#111](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/111)
* Bump versions from 0.1.1 to 0.1.2 by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#109](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/109)
* Set up trusted publishing for Postgres.js by [@danielfrankcom](https://github.com/danielfrankcom) in [awslabs/aurora-dsql-nodejs-connector#116](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/116)

## New Contributors
* [@danielfrankcom](https://github.com/danielfrankcom) made their first contribution in [awslabs/aurora-dsql-nodejs-connector#92](https://github.com/awslabs/aurora-dsql-nodejs-connector/pull/92)

**Full Changelog**: https://github.com/awslabs/aurora-dsql-connectors/compare/node/postgres-js/v0.1.1...node/postgres-js/v0.1.2



[Changes][node/postgres-js/v0.1.2]


<a id="node/postgres-js/v0.1.1"></a>
# [Aurora DSQL Connector for Postgres.js v0.1.1 (node/postgres-js/v0.1.1)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/node/postgres-js/v0.1.1) - 2026-02-04

> **Note:** This release was originally published on Nov 26, 2025 by [@mitchell-elholm](https://github.com/mitchell-elholm) in [awslabs/aurora-dsql-nodejs-connector](https://github.com/awslabs/aurora-dsql-nodejs-connector/releases/tag/aurora-dsql-postgresjs-connector-v0.1.1).

---

## What's Changed
- Bug fix for relative imports



[Changes][node/postgres-js/v0.1.1]


<a id="node/postgres-js/v0.1.0"></a>
# [Aurora DSQL Connector for Postgres.js v0.1.0 (node/postgres-js/v0.1.0)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/node/postgres-js/v0.1.0) - 2026-02-04

> **Note:** This release was originally published on Oct 30, 2025 by [@mitchell-elholm](https://github.com/mitchell-elholm) in [awslabs/aurora-dsql-nodejs-connector](https://github.com/awslabs/aurora-dsql-nodejs-connector/releases/tag/aurora-dsql-postgresjs-connector-v0.1.0).

---

Initial release of Aurora DSQL Connector for Postgres.js



[Changes][node/postgres-js/v0.1.0]


[node/postgres-js/v0.2.1]: https://github.com/awslabs/aurora-dsql-connectors/compare/node/postgres-js/v0.2.0...node/postgres-js/v0.2.1
[node/postgres-js/v0.2.0]: https://github.com/awslabs/aurora-dsql-connectors/compare/node/postgres-js/v0.1.3...node/postgres-js/v0.2.0
[node/postgres-js/v0.1.3]: https://github.com/awslabs/aurora-dsql-connectors/compare/node/postgres-js/v0.1.2...node/postgres-js/v0.1.3
[node/postgres-js/v0.1.2]: https://github.com/awslabs/aurora-dsql-connectors/compare/node/postgres-js/v0.1.1...node/postgres-js/v0.1.2
[node/postgres-js/v0.1.1]: https://github.com/awslabs/aurora-dsql-connectors/compare/node/postgres-js/v0.1.0...node/postgres-js/v0.1.1
[node/postgres-js/v0.1.0]: https://github.com/awslabs/aurora-dsql-connectors/tree/node/postgres-js/v0.1.0

<!-- Generated by https://github.com/rhysd/changelog-from-release v3.9.1 -->
