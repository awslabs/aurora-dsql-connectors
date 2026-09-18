/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

package occretry

import (
	"context"
	"errors"
	"math"
	"strings"
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

func TestRetry_PartialConfigUsesDefaults(t *testing.T) {
	calls := 0
	err := Retry(context.Background(), Config{MaxRetries: 2}, func() error {
		calls++
		if calls < 3 {
			return &pgconn.PgError{Code: ErrorCodeMutation, Message: "conflict"}
		}
		return nil
	})

	if err != nil {
		t.Fatalf("expected retry to succeed, got %v", err)
	}
	if calls != 3 {
		t.Fatalf("expected 3 attempts, got %d", calls)
	}
}

func TestRetry_ZeroRetriesExecutesOnce(t *testing.T) {
	calls := 0
	err := Retry(context.Background(), Config{}, func() error {
		calls++
		return &pgconn.PgError{Code: ErrorCodeMutation, Message: "conflict"}
	})

	if err == nil {
		t.Fatal("expected OCC error")
	}
	if calls != 1 {
		t.Fatalf("expected 1 attempt, got %d", calls)
	}
}

func TestRetry_InvalidConfigReturnsBeforeExecuting(t *testing.T) {
	tests := []struct {
		name       string
		config     Config
		wantErrMsg string
	}{
		{
			name:       "negative max retries",
			config:     Config{MaxRetries: -1},
			wantErrMsg: "max retries must be non-negative",
		},
		{
			name:       "negative initial wait with retries disabled",
			config:     Config{InitialWait: -time.Millisecond},
			wantErrMsg: "initial wait must be positive",
		},
		{
			name: "max wait below initial wait",
			config: Config{
				MaxRetries:  1,
				InitialWait: 2 * time.Second,
				MaxWait:     time.Second,
				Multiplier:  2,
			},
			wantErrMsg: "max wait must be at least initial wait",
		},
		{
			name: "multiplier below one",
			config: Config{
				MaxRetries:  1,
				InitialWait: time.Millisecond,
				MaxWait:     time.Second,
				Multiplier:  0.5,
			},
			wantErrMsg: "multiplier must be finite and at least 1",
		},
		{
			name: "multiplier is NaN",
			config: Config{
				MaxRetries:  1,
				InitialWait: time.Millisecond,
				MaxWait:     time.Second,
				Multiplier:  math.NaN(),
			},
			wantErrMsg: "multiplier must be finite and at least 1",
		},
		{
			name: "multiplier is infinite",
			config: Config{
				MaxRetries:  1,
				InitialWait: time.Millisecond,
				MaxWait:     time.Second,
				Multiplier:  math.Inf(1),
			},
			wantErrMsg: "multiplier must be finite and at least 1",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			calls := 0
			err := Retry(context.Background(), tt.config, func() error {
				calls++
				return nil
			})

			if err == nil {
				t.Fatal("expected invalid configuration error")
			}
			if !strings.Contains(err.Error(), tt.wantErrMsg) {
				t.Fatalf("expected error containing %q, got %v", tt.wantErrMsg, err)
			}
			if calls != 0 {
				t.Fatalf("expected no attempts with invalid configuration, got %d", calls)
			}
		})
	}
}

func TestBackoffWait_OneNanosecondWaitDoesNotPanic(t *testing.T) {
	wait := time.Nanosecond
	nextWait, err := backoffWait(context.Background(), wait, Config{
		MaxWait:    10 * time.Nanosecond,
		Multiplier: 2,
	})
	if err != nil {
		t.Fatalf("expected nil error, got %v", err)
	}
	if nextWait != 2*time.Nanosecond {
		t.Fatalf("expected next wait %s, got %s", 2*time.Nanosecond, nextWait)
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
