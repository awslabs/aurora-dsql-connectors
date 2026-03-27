/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

package occretry

import (
	"context"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
)

// DB provides pgx-style database operations with automatic OCC retry.
// Use [New] to create a DB that wraps a *pgxpool.Pool (or any type satisfying
// the required interfaces) with retry logic.
//
// For operations that don't need retry, use the underlying pool directly or
// pass a context wrapped with [NoRetry] to skip retry for a single call.
type DB interface {
	// Exec executes a SQL statement with automatic OCC retry.
	// On OCC conflict the statement is re-executed. Ensure the statement
	// has no non-transactional side effects that should not be repeated
	// (e.g., sending notifications, enqueuing messages).
	Exec(ctx context.Context, sql string, arguments ...any) (pgconn.CommandTag, error)

	// Query executes a query with automatic OCC retry.
	// Only errors returned by the Query call itself are retried;
	// errors surfaced during row iteration (via rows.Next or rows.Err)
	// are not.
	Query(ctx context.Context, sql string, args ...any) (pgx.Rows, error)

	// QueryRow executes a query returning a single row.
	// QueryRow delegates directly to the underlying pool without retry because
	// pgx.Row defers errors to Scan. For retryable single-row reads, use
	// [DB.WithTransaction].
	QueryRow(ctx context.Context, sql string, args ...any) pgx.Row

	// WithTransaction executes fn in a transaction with automatic OCC retry.
	// On OCC conflict (whether during the callback or at commit), the
	// transaction is rolled back and fn is re-executed from scratch.
	// Ensure fn contains only database operations
	// and has no side effects that should not be repeated (e.g., sending
	// notifications, enqueuing messages).
	// The caller must not call Commit or Rollback — they are managed
	// automatically.
	WithTransaction(ctx context.Context, fn func(tx pgx.Tx) error) error
}

// noRetryKey is the context key for opting out of retry on a per-call basis.
type noRetryKey struct{}

// NoRetry returns a context that disables OCC retry for a single DB call.
//
// Example:
//
//	db.Exec(occretry.NoRetry(ctx), "SELECT 1")  // not retried
//	db.Exec(ctx, "CREATE TABLE t (...)")         // retried
func NoRetry(ctx context.Context) context.Context {
	return context.WithValue(ctx, noRetryKey{}, true)
}

func isNoRetry(ctx context.Context) bool {
	v, _ := ctx.Value(noRetryKey{}).(bool)
	return v
}

// pool defines the interface that *pgxpool.Pool satisfies for the operations
// needed by the retry-aware DB. It composes the existing Execer and Beginner
// interfaces with query methods.
type pool interface {
	Execer
	Beginner
	Query(ctx context.Context, sql string, args ...any) (pgx.Rows, error)
	QueryRow(ctx context.Context, sql string, args ...any) pgx.Row
}

type retryDB struct {
	pool   pool
	config Config
}

// New creates a [DB] that wraps the given pool with automatic OCC retry.
// The pool is typically a *pgxpool.Pool created via dsql.NewPool.
//
// Example:
//
//	pool, _ := dsql.NewPool(ctx, dsql.Config{Host: "a1b2c3d4e5f6g7h8i9j0klmnop.dsql.us-east-1.on.aws"})
//	db := occretry.New(pool, occretry.DefaultConfig())
//
//	// Retried automatically
//	db.Exec(ctx, "CREATE TABLE t (id UUID PRIMARY KEY)")
//
//	// Opt out for a single call
//	db.Exec(occretry.NoRetry(ctx), "SELECT 1")
func New(p pool, config Config) DB {
	if p == nil {
		panic("occretry.New: pool must not be nil")
	}
	return &retryDB{pool: p, config: config}
}

func (r *retryDB) Exec(ctx context.Context, sql string, arguments ...any) (pgconn.CommandTag, error) {
	if isNoRetry(ctx) {
		return r.pool.Exec(ctx, sql, arguments...)
	}
	var tag pgconn.CommandTag
	err := Retry(ctx, r.config, func() error {
		var execErr error
		tag, execErr = r.pool.Exec(ctx, sql, arguments...)
		return execErr
	})
	return tag, err
}

func (r *retryDB) Query(ctx context.Context, sql string, args ...any) (pgx.Rows, error) {
	if isNoRetry(ctx) {
		return r.pool.Query(ctx, sql, args...)
	}
	var rows pgx.Rows
	err := Retry(ctx, r.config, func() error {
		var queryErr error
		rows, queryErr = r.pool.Query(ctx, sql, args...)
		return queryErr
	})
	return rows, err
}

func (r *retryDB) QueryRow(ctx context.Context, sql string, args ...any) pgx.Row {
	return r.pool.QueryRow(ctx, sql, args...)
}

func (r *retryDB) WithTransaction(ctx context.Context, fn func(tx pgx.Tx) error) error {
	if isNoRetry(ctx) {
		tx, err := r.pool.Begin(ctx)
		if err != nil {
			return err
		}
		defer tx.Rollback(ctx)
		if err := fn(tx); err != nil {
			return err
		}
		return tx.Commit(ctx)
	}
	return WithRetry(ctx, r.pool, r.config, fn)
}
