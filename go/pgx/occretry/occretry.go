/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

// Package occretry provides utilities for handling Optimistic Concurrency Control (OCC)
// conflicts in Aurora DSQL.
//
// Aurora DSQL uses OCC where conflicts are detected at commit time. When two
// transactions modify the same data, the first to commit wins and the second
// receives an OCC error (codes "OC000" or "OC001"). Applications should retry
// failed transactions with exponential backoff.
//
// Key concepts:
//   - OCC allows high concurrency by not locking rows during reads
//   - Conflicts are only detected at commit time
//   - Failed transactions should be retried with fresh data
//   - Use exponential backoff with jitter to avoid thundering herd
//
// For more information, see:
// https://aws.amazon.com/blogs/database/concurrency-control-in-amazon-aurora-dsql/
package occretry

import (
	"context"
	"errors"
	"fmt"
	"math/rand"
	"time"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
)

// Beginner is an interface for types that can begin a database transaction.
// *pgxpool.Pool satisfies this interface.
type Beginner interface {
	Begin(ctx context.Context) (pgx.Tx, error)
}

// OCC error codes for Aurora DSQL optimistic concurrency control conflicts.
const (
	// ErrorCodeMutation is returned when a mutation conflicts with another transaction.
	// This occurs when two transactions try to modify the same rows concurrently.
	ErrorCodeMutation = "OC000"

	// ErrorCodeSchema is returned when a schema change conflicts with the transaction.
	// This occurs after DDL operations (CREATE TABLE, ALTER TABLE, etc.) when
	// subsequent transactions see the schema change.
	ErrorCodeSchema = "OC001"
)

// Config holds configuration for retry behavior.
type Config struct {
	// MaxRetries is the maximum number of retry attempts (default: 3)
	MaxRetries int

	// InitialWait is the initial wait duration before first retry (default: 100ms)
	InitialWait time.Duration

	// MaxWait is the maximum wait duration between retries (default: 5s)
	MaxWait time.Duration

	// Multiplier is the exponential backoff multiplier (default: 2.0)
	Multiplier float64
}

// DefaultConfig returns sensible defaults for DSQL OCC retry.
func DefaultConfig() Config {
	return Config{
		MaxRetries:  3,
		InitialWait: 100 * time.Millisecond,
		MaxWait:     5 * time.Second,
		Multiplier:  2.0,
	}
}

// IsOCCError checks if an error is a DSQL OCC conflict error.
// Returns true for OC000 (mutation conflict), OC001 (schema conflict),
// and 40001 (serialization failure) errors.
func IsOCCError(err error) bool {
	if err == nil {
		return false
	}
	var pgErr *pgconn.PgError
	if errors.As(err, &pgErr) {
		return pgErr.Code == ErrorCodeMutation || pgErr.Code == ErrorCodeSchema || pgErr.Code == "40001"
	}
	return false
}

// backoffWait waits with exponential backoff and jitter. Returns the next wait duration.
func backoffWait(ctx context.Context, wait time.Duration, config Config) (time.Duration, error) {
	jitter := time.Duration(rand.Int63n(int64(wait / 4)))
	sleepTime := wait + jitter

	// Use select to allow cancellation during the backoff wait.
	// ctx.Done() returns a channel that closes when the context is cancelled.
	select {
	case <-ctx.Done():
		return 0, ctx.Err()
	case <-time.After(sleepTime):
	}

	nextWait := time.Duration(float64(wait) * config.Multiplier)
	if nextWait > config.MaxWait {
		nextWait = config.MaxWait
	}
	return nextWait, nil
}

// Retry executes fn with automatic retry on OCC conflicts.
// This is the core retry primitive — fn can be any operation that may encounter
// OCC errors. If fn returns an OCC error, it is retried with exponential backoff.
// Non-OCC errors are returned immediately.
//
// Example:
//
//	err := occretry.Retry(ctx, occretry.DefaultConfig(), func() error {
//	    _, err := pool.Exec(ctx, "INSERT INTO users (id, name) VALUES ($1, $2)", id, name)
//	    return err
//	})
func Retry(ctx context.Context, config Config, fn func() error) error {
	var lastErr error
	wait := config.InitialWait

	for attempt := 0; attempt <= config.MaxRetries; attempt++ {
		err := fn()
		if err == nil {
			return nil
		}

		if !IsOCCError(err) {
			return err
		}

		lastErr = err

		// Wait before next retry (skip on last attempt)
		if attempt < config.MaxRetries {
			var waitErr error
			wait, waitErr = backoffWait(ctx, wait, config)
			if waitErr != nil {
				return waitErr
			}
		}
	}

	return fmt.Errorf("max retries (%d) exceeded, last error: %w", config.MaxRetries, lastErr)
}

// Execer is an interface for types that can execute SQL statements.
// *pgxpool.Pool and *pgx.Conn satisfy this interface.
type Execer interface {
	Exec(ctx context.Context, sql string, arguments ...any) (pgconn.CommandTag, error)
}

// ExecWithRetry executes a single SQL statement with automatic retry on OCC conflicts.
// Unlike WithRetry, this does NOT wrap in an explicit transaction, making it suitable
// for both DDL (CREATE TABLE, CREATE INDEX ASYNC, etc.) and single DML statements.
//
// Example:
//
//	err := occretry.ExecWithRetry(ctx, pool, occretry.DefaultConfig(),
//	    "CREATE INDEX ASYNC ON users (email)")
func ExecWithRetry(ctx context.Context, execer Execer, config Config, sql string, arguments ...any) error {
	return Retry(ctx, config, func() error {
		_, err := execer.Exec(ctx, sql, arguments...)
		return err
	})
}

// WithRetry executes a transactional function with automatic retry on OCC conflicts.
// It begins a transaction, calls fn with the transaction, and commits on success.
// If an OCC error occurs at any point, the transaction is rolled back and retried
// with exponential backoff.
//
// Example:
//
//	err := occretry.WithRetry(ctx, pool, occretry.DefaultConfig(), func(tx pgx.Tx) error {
//	    _, err := tx.Exec(ctx, "UPDATE accounts SET balance = balance - $1 WHERE id = $2", amount, fromID)
//	    if err != nil {
//	        return err
//	    }
//	    _, err = tx.Exec(ctx, "UPDATE accounts SET balance = balance + $1 WHERE id = $2", amount, toID)
//	    return err
//	})
//
// To return values from the transaction, use a closure to capture results:
//
//	var balance int
//	err := occretry.WithRetry(ctx, pool, occretry.DefaultConfig(), func(tx pgx.Tx) error {
//	    return tx.QueryRow(ctx, "SELECT balance FROM accounts WHERE id = $1", id).Scan(&balance)
//	})
//	// balance is now set if err == nil
func WithRetry(ctx context.Context, pool Beginner, config Config, fn func(tx pgx.Tx) error) error {
	return Retry(ctx, config, func() error {
		tx, err := pool.Begin(ctx)
		if err != nil {
			return fmt.Errorf("begin transaction: %w", err)
		}
		defer tx.Rollback(ctx) // No-op if committed

		if err := fn(tx); err != nil {
			return err
		}

		return tx.Commit(ctx)
	})
}
