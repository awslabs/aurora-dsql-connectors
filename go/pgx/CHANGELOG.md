<a id="go/pgx/v0.5.0"></a>
# [Aurora DSQL Connector for Go pgx v0.5.0 (go/pgx/v0.5.0)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/go/pgx/v0.5.0) - 2026-09-04

## What's Changed

- Default the pgx query execution mode to `QueryExecModeDescribeExec` so pooled connections remain safe across live schema changes.
- Add `Config.QueryExecMode` for callers that need to select another pgx execution mode.
- Document the execution-mode correctness and latency tradeoffs, plus guidance for evolving schemas.
- Refresh AWS SDK dependencies and documentation links since `v0.4.0`.

Thanks to [@rmconstantin](https://github.com/rmconstantin) for the implementation and live-cluster validation, and [@praba2210](https://github.com/praba2210) for the detailed review.

**Full Changelog**: https://github.com/awslabs/aurora-dsql-connectors/compare/go/pgx/v0.4.0...go/pgx/v0.5.0


[Changes][go/pgx/v0.5.0]


<a id="go/pgx/v0.4.0"></a>
# [Aurora DSQL Connector for Go pgx v0.4.0 (go/pgx/v0.4.0)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/go/pgx/v0.4.0) - 2026-10-02

_Tagged on 2026-03-30. This GitHub Release was added later._

## What's Changed

- Add DB interface for pool-level OCC retry in Go connector ([#322](https://github.com/awslabs/aurora-dsql-connectors/issues/322))
- Remove per-connector scaffolding that duplicates repo root ([#280](https://github.com/awslabs/aurora-dsql-connectors/issues/280))
- Add ExecWithRetry helper to Go OCC retry package ([#271](https://github.com/awslabs/aurora-dsql-connectors/issues/271))

**Full Changelog**: https://github.com/awslabs/aurora-dsql-connectors/compare/go/pgx/v0.3.0...go/pgx/v0.4.0


[Changes][go/pgx/v0.4.0]


<a id="go/pgx/v0.3.0"></a>
# [Aurora DSQL Connector for Go pgx v0.3.0 (go/pgx/v0.3.0)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/go/pgx/v0.3.0) - 2026-03-18

## Breaking Changes

- **Removed `dsql.Pool` wrapper type** — `dsql.NewPool` now returns `*pgxpool.Pool` directly
- **Removed `dsql.Conn` wrapper type** — `dsql.Connect` now returns `*pgx.Conn` directly
- **Removed `occretry.ExecWithRetry`** — use `occretry.Retry` with inline `pool.Exec()` instead
- **`occretry.WithRetry`** now accepts `occretry.Beginner` interface instead of `*dsql.Pool`
- **`occretry.IsOCCError`** no longer matches OCC codes via string matching — only via `*pgconn.PgError` error codes

## New

- **`occretry.Retry(ctx, config, fn func() error)`** — core retry primitive that decouples retry logic from transaction management
- **`occretry.Beginner` interface** — any type with `Begin(ctx) (pgx.Tx, error)` works, including `*pgxpool.Pool`

## Improvements

- `occretry` package no longer depends on `dsql` — only depends on `pgx/v5` and `pgconn`
- `occretry.WithRetry` is now a thin wrapper around `Retry`, reducing code duplication
- `IsOCCError` uses `errors.As` with `*pgconn.PgError` for robust error detection

## Migration

```go
// Before: dsql.Pool wrapper
pool, err := dsql.NewPool(ctx, dsql.Config{...})
pool.Pool // had to unwrap

// After: standard pgxpool.Pool
pool, err := dsql.NewPool(ctx, dsql.Config{...})
// use pool directly

// Before: ExecWithRetry
occretry.ExecWithRetry(ctx, pool, "CREATE TABLE ...", 5)

// After: Retry
config := occretry.DefaultConfig()
config.MaxRetries = 5
occretry.Retry(ctx, config, func() error {
    _, err := pool.Exec(ctx, "CREATE TABLE ...")
    return err
})
```

[Changes][go/pgx/v0.3.0]


<a id="go/pgx/v0.2.0"></a>
# [Aurora DSQL Connector for Go pgx v0.2.0 (go/pgx/v0.2.0)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/go/pgx/v0.2.0) - 2026-10-02

_Tagged on 2026-03-13. This GitHub Release was added later._

## What's Changed

- Accept pgxpool.Config directly in NewPool ([#209](https://github.com/awslabs/aurora-dsql-connectors/issues/209))
- Fix incorrect max token duration claim in Go connector ([#208](https://github.com/awslabs/aurora-dsql-connectors/issues/208))

### Dependency Updates

- 1 dependency update(s).

**Full Changelog**: https://github.com/awslabs/aurora-dsql-connectors/compare/go/pgx/v0.1.3...go/pgx/v0.2.0


[Changes][go/pgx/v0.2.0]


<a id="go/pgx/v0.1.3"></a>
# [Aurora DSQL Connector for Go pgx v0.1.3 (go/pgx/v0.1.3)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/go/pgx/v0.1.3) - 2026-03-03

## What's Changed

### Breaking Changes
- Removed token caching from connection pool. Token generation is a local SigV4 presigning operation with no network overhead, so caching added complexity for zero benefit. The `TokenCache` field and `ClearTokenCache()` method have been removed from `Pool`.

### Maintenance
- Updated AWS SDK Go v2 dependencies to latest patch versions

### Full Changelog
https://github.com/awslabs/aurora-dsql-connectors/compare/go/pgx/v0.1.2...go/pgx/v0.1.3

[Changes][go/pgx/v0.1.3]


<a id="go/pgx/v0.1.2"></a>
# [Aurora DSQL Connector for Go pgx v0.1.2 (go/pgx/v0.1.2)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/go/pgx/v0.1.2) - 2026-10-02

_Tagged on 2026-02-03. This GitHub Release was added later._

## What's Changed

- Standardize CI/CD workflows for monorepo structure
- Fix CodeQL port parse warning in Go pgx connector ([#354](https://github.com/awslabs/aurora-dsql-connectors/issues/354))
- feat(go): add DSQL connector enhancements and OCC retry utilities ([#347](https://github.com/awslabs/aurora-dsql-connectors/issues/347))
- feat(go): add transaction, OCC retry, and connection string examples ([#319](https://github.com/awslabs/aurora-dsql-connectors/issues/319))
- chore(go-connector): update example to use published v0.1.0 module ([#316](https://github.com/awslabs/aurora-dsql-connectors/issues/316))
- refactor(go-connector): consolidate config validation for fail-fast behavior ([#315](https://github.com/awslabs/aurora-dsql-connectors/issues/315))
- fix(go-connector): address post-merge review comments from PR [#306](https://github.com/awslabs/aurora-dsql-connectors/issues/306) ([#314](https://github.com/awslabs/aurora-dsql-connectors/issues/314))
- feat: Add Go connector for Aurora DSQL using pgx ([#306](https://github.com/awslabs/aurora-dsql-connectors/issues/306))
- Remove standalone directories for monorepo restructure
- Add go pgx connector

**Full Changelog**: https://github.com/awslabs/aurora-dsql-connectors/compare/go/pgx/v0.1.1...go/pgx/v0.1.2


[Changes][go/pgx/v0.1.2]


<a id="go/pgx/v0.1.1"></a>
# [Aurora DSQL Connector for Go pgx v0.1.1 (go/pgx/v0.1.1)](https://github.com/awslabs/aurora-dsql-connectors/releases/tag/go/pgx/v0.1.1) - 2026-10-02

_Tagged on 2026-01-20. This GitHub Release was added later._

## What's Changed

- feat(go): add DSQL connector enhancements and OCC retry utilities ([#347](https://github.com/awslabs/aurora-dsql-connectors/issues/347))
- feat(go): add transaction, OCC retry, and connection string examples ([#319](https://github.com/awslabs/aurora-dsql-connectors/issues/319))
- chore(go-connector): update example to use published v0.1.0 module ([#316](https://github.com/awslabs/aurora-dsql-connectors/issues/316))
- refactor(go-connector): consolidate config validation for fail-fast behavior ([#315](https://github.com/awslabs/aurora-dsql-connectors/issues/315))
- fix(go-connector): address post-merge review comments from PR [#306](https://github.com/awslabs/aurora-dsql-connectors/issues/306) ([#314](https://github.com/awslabs/aurora-dsql-connectors/issues/314))
- feat: Add Go connector for Aurora DSQL using pgx ([#306](https://github.com/awslabs/aurora-dsql-connectors/issues/306))


[Changes][go/pgx/v0.1.1]


[go/pgx/v0.5.0]: https://github.com/awslabs/aurora-dsql-connectors/compare/go/pgx/v0.4.0...go/pgx/v0.5.0
[go/pgx/v0.4.0]: https://github.com/awslabs/aurora-dsql-connectors/compare/go/pgx/v0.3.0...go/pgx/v0.4.0
[go/pgx/v0.3.0]: https://github.com/awslabs/aurora-dsql-connectors/compare/go/pgx/v0.2.0...go/pgx/v0.3.0
[go/pgx/v0.2.0]: https://github.com/awslabs/aurora-dsql-connectors/compare/go/pgx/v0.1.3...go/pgx/v0.2.0
[go/pgx/v0.1.3]: https://github.com/awslabs/aurora-dsql-connectors/compare/go/pgx/v0.1.2...go/pgx/v0.1.3
[go/pgx/v0.1.2]: https://github.com/awslabs/aurora-dsql-connectors/compare/go/pgx/v0.1.1...go/pgx/v0.1.2
[go/pgx/v0.1.1]: https://github.com/awslabs/aurora-dsql-connectors/tree/go/pgx/v0.1.1

<!-- Generated by https://github.com/rhysd/changelog-from-release v3.9.1 -->
