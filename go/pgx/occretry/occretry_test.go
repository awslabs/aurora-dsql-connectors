/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

package occretry

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/jackc/pgx/v5/pgconn"
)

// mockExecer is a test double that records calls and returns preset results.
type mockExecer struct {
	calls     int
	returnErr error
	// errs, if set, is used instead of returnErr to return different errors per call.
	errs []error
}

func (m *mockExecer) Exec(ctx context.Context, sql string, arguments ...any) (pgconn.CommandTag, error) {
	idx := m.calls
	m.calls++
	if m.errs != nil && idx < len(m.errs) {
		return pgconn.CommandTag{}, m.errs[idx]
	}
	return pgconn.CommandTag{}, m.returnErr
}

func fastConfig() Config {
	return Config{
		MaxRetries:  3,
		InitialWait: 1 * time.Millisecond,
		MaxWait:     5 * time.Millisecond,
		Multiplier:  2.0,
	}
}

func TestExecWithRetry_Success(t *testing.T) {
	mock := &mockExecer{}
	err := ExecWithRetry(context.Background(), mock, fastConfig(), "INSERT INTO t VALUES ($1)", 1)
	if err != nil {
		t.Fatalf("expected nil error, got %v", err)
	}
	if mock.calls != 1 {
		t.Fatalf("expected 1 call, got %d", mock.calls)
	}
}

func TestExecWithRetry_OCCErrorTriggersRetry(t *testing.T) {
	occErr := &pgconn.PgError{Code: "OC000", Message: "transaction conflict"}
	mock := &mockExecer{
		errs: []error{occErr, occErr, nil},
	}
	err := ExecWithRetry(context.Background(), mock, fastConfig(), "UPDATE t SET x = 1")
	if err != nil {
		t.Fatalf("expected nil error after retries, got %v", err)
	}
	if mock.calls != 3 {
		t.Fatalf("expected 3 calls (2 failures + 1 success), got %d", mock.calls)
	}
}

func TestExecWithRetry_NonOCCErrorReturnsImmediately(t *testing.T) {
	nonOCCErr := errors.New("connection refused")
	mock := &mockExecer{returnErr: nonOCCErr}
	err := ExecWithRetry(context.Background(), mock, fastConfig(), "SELECT 1")
	if !errors.Is(err, nonOCCErr) {
		t.Fatalf("expected non-OCC error to be returned immediately, got %v", err)
	}
	if mock.calls != 1 {
		t.Fatalf("expected 1 call (no retry for non-OCC error), got %d", mock.calls)
	}
}

func TestExecWithRetry_ExhaustsRetries(t *testing.T) {
	occErr := &pgconn.PgError{Code: "OC001", Message: "schema conflict"}
	mock := &mockExecer{returnErr: occErr}
	err := ExecWithRetry(context.Background(), mock, fastConfig(), "CREATE INDEX ASYNC ON t (col)")
	if err == nil {
		t.Fatal("expected error after exhausting retries, got nil")
	}
	// MaxRetries is 3, so we expect 1 initial + 3 retries = 4 calls
	if mock.calls != 4 {
		t.Fatalf("expected 4 calls (1 initial + 3 retries), got %d", mock.calls)
	}
}

func TestExecWithRetry_PassesArguments(t *testing.T) {
	var capturedSQL string
	var capturedArgs []any
	wrapper := &argCapturingExecer{
		onExec: func(sql string, args []any) {
			capturedSQL = sql
			capturedArgs = args
		},
	}
	err := ExecWithRetry(context.Background(), wrapper, fastConfig(),
		"INSERT INTO t (a, b) VALUES ($1, $2)", "hello", 42)
	if err != nil {
		t.Fatalf("expected nil error, got %v", err)
	}
	if capturedSQL != "INSERT INTO t (a, b) VALUES ($1, $2)" {
		t.Fatalf("unexpected sql: %s", capturedSQL)
	}
	if len(capturedArgs) != 2 || capturedArgs[0] != "hello" || capturedArgs[1] != 42 {
		t.Fatalf("unexpected arguments: %v", capturedArgs)
	}
}

type argCapturingExecer struct {
	onExec func(sql string, args []any)
}

func (a *argCapturingExecer) Exec(ctx context.Context, sql string, arguments ...any) (pgconn.CommandTag, error) {
	if a.onExec != nil {
		a.onExec(sql, arguments)
	}
	return pgconn.CommandTag{}, nil
}
