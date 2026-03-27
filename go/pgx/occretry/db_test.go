/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

package occretry

import (
	"context"
	"errors"
	"testing"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
)

// mockPool implements the pool interface for testing DB methods.
type mockPool struct {
	execCalls     int
	execErrs      []error
	queryCalls    int
	queryErrs     []error
	queryRowCalls int
	beginCalls    int
	txSequence    []*mockTx
}

func (m *mockPool) Exec(ctx context.Context, sql string, arguments ...any) (pgconn.CommandTag, error) {
	idx := m.execCalls
	m.execCalls++
	if idx < len(m.execErrs) {
		return pgconn.CommandTag{}, m.execErrs[idx]
	}
	return pgconn.CommandTag{}, nil
}

func (m *mockPool) Query(ctx context.Context, sql string, args ...any) (pgx.Rows, error) {
	idx := m.queryCalls
	m.queryCalls++
	if idx < len(m.queryErrs) {
		return nil, m.queryErrs[idx]
	}
	return nil, nil
}

func (m *mockPool) QueryRow(ctx context.Context, sql string, args ...any) pgx.Row {
	m.queryRowCalls++
	return nil
}

func (m *mockPool) Begin(ctx context.Context) (pgx.Tx, error) {
	idx := m.beginCalls
	m.beginCalls++
	if idx < len(m.txSequence) {
		return m.txSequence[idx], nil
	}
	if len(m.txSequence) > 0 {
		return m.txSequence[len(m.txSequence)-1], nil
	}
	return &mockTx{}, nil
}

// mockTx implements pgx.Tx for testing.
type mockTx struct {
	commitErr     error
	commitCalls   int
	rollbackCalls int
	execCalls     int
}

func (m *mockTx) Begin(ctx context.Context) (pgx.Tx, error) { return nil, nil }
func (m *mockTx) Commit(ctx context.Context) error {
	m.commitCalls++
	return m.commitErr
}
func (m *mockTx) Rollback(ctx context.Context) error { m.rollbackCalls++; return nil }
func (m *mockTx) Exec(ctx context.Context, sql string, args ...any) (pgconn.CommandTag, error) {
	m.execCalls++
	return pgconn.CommandTag{}, nil
}
func (m *mockTx) Query(ctx context.Context, sql string, args ...any) (pgx.Rows, error) {
	return nil, nil
}
func (m *mockTx) QueryRow(ctx context.Context, sql string, args ...any) pgx.Row { return nil }
func (m *mockTx) CopyFrom(ctx context.Context, tableName pgx.Identifier, columnNames []string, rowSrc pgx.CopyFromSource) (int64, error) {
	return 0, nil
}
func (m *mockTx) SendBatch(ctx context.Context, b *pgx.Batch) pgx.BatchResults { return nil }
func (m *mockTx) LargeObjects() pgx.LargeObjects                               { return pgx.LargeObjects{} }
func (m *mockTx) Prepare(ctx context.Context, name, sql string) (*pgconn.StatementDescription, error) {
	return nil, nil
}
func (m *mockTx) Conn() *pgx.Conn { return nil }

func newOCCError(code string) error {
	return &pgconn.PgError{Code: code, Message: "conflict"}
}

// --- Exec tests ---

func TestDB_Exec_Success(t *testing.T) {
	mock := &mockPool{}
	db := New(mock, fastConfig())
	_, err := db.Exec(context.Background(), "CREATE TABLE t (id UUID PRIMARY KEY)")
	if err != nil {
		t.Fatalf("expected nil error, got %v", err)
	}
	if mock.execCalls != 1 {
		t.Fatalf("expected 1 call, got %d", mock.execCalls)
	}
}

func TestDB_Exec_RetriesOnOCC(t *testing.T) {
	mock := &mockPool{
		execErrs: []error{newOCCError("OC000"), newOCCError("OC000"), nil},
	}
	db := New(mock, fastConfig())
	_, err := db.Exec(context.Background(), "INSERT INTO t VALUES ($1)", 1)
	if err != nil {
		t.Fatalf("expected nil error after retries, got %v", err)
	}
	if mock.execCalls != 3 {
		t.Fatalf("expected 3 calls, got %d", mock.execCalls)
	}
}

func TestDB_Exec_NonOCCErrorReturnsImmediately(t *testing.T) {
	nonOCCErr := errors.New("connection refused")
	mock := &mockPool{execErrs: []error{nonOCCErr}}
	db := New(mock, fastConfig())
	_, err := db.Exec(context.Background(), "SELECT 1")
	if !errors.Is(err, nonOCCErr) {
		t.Fatalf("expected non-OCC error, got %v", err)
	}
	if mock.execCalls != 1 {
		t.Fatalf("expected 1 call, got %d", mock.execCalls)
	}
}

func TestDB_Exec_ExhaustsRetries(t *testing.T) {
	mock := &mockPool{
		execErrs: []error{
			newOCCError("OC001"), newOCCError("OC001"),
			newOCCError("OC001"), newOCCError("OC001"),
		},
	}
	db := New(mock, fastConfig())
	_, err := db.Exec(context.Background(), "CREATE INDEX ASYNC ON t (col)")
	if err == nil {
		t.Fatal("expected error after exhausting retries")
	}
	// 1 initial + 3 retries = 4
	if mock.execCalls != 4 {
		t.Fatalf("expected 4 calls, got %d", mock.execCalls)
	}
}

// --- Exec NoRetry tests ---

func TestDB_Exec_NoRetrySkipsRetry(t *testing.T) {
	mock := &mockPool{
		execErrs: []error{newOCCError("OC000")},
	}
	db := New(mock, fastConfig())
	ctx := NoRetry(context.Background())
	_, err := db.Exec(ctx, "INSERT INTO t VALUES ($1)", 1)
	if err == nil {
		t.Fatal("expected OCC error to be returned without retry")
	}
	if mock.execCalls != 1 {
		t.Fatalf("expected 1 call (no retry), got %d", mock.execCalls)
	}
}

func TestDB_Exec_NoRetrySuccess(t *testing.T) {
	mock := &mockPool{}
	db := New(mock, fastConfig())
	ctx := NoRetry(context.Background())
	_, err := db.Exec(ctx, "SELECT 1")
	if err != nil {
		t.Fatalf("expected nil error, got %v", err)
	}
	if mock.execCalls != 1 {
		t.Fatalf("expected 1 call, got %d", mock.execCalls)
	}
}

// --- Query tests ---

func TestDB_Query_Success(t *testing.T) {
	mock := &mockPool{}
	db := New(mock, fastConfig())
	_, err := db.Query(context.Background(), "SELECT * FROM t")
	if err != nil {
		t.Fatalf("expected nil error, got %v", err)
	}
	if mock.queryCalls != 1 {
		t.Fatalf("expected 1 call, got %d", mock.queryCalls)
	}
}

func TestDB_Query_RetriesOnOCC(t *testing.T) {
	mock := &mockPool{
		queryErrs: []error{newOCCError("OC000"), nil},
	}
	db := New(mock, fastConfig())
	_, err := db.Query(context.Background(), "SELECT * FROM t")
	if err != nil {
		t.Fatalf("expected nil error, got %v", err)
	}
	if mock.queryCalls != 2 {
		t.Fatalf("expected 2 calls, got %d", mock.queryCalls)
	}
}

func TestDB_Query_NonOCCErrorReturnsImmediately(t *testing.T) {
	nonOCCErr := errors.New("connection refused")
	mock := &mockPool{queryErrs: []error{nonOCCErr}}
	db := New(mock, fastConfig())
	_, err := db.Query(context.Background(), "SELECT * FROM t")
	if !errors.Is(err, nonOCCErr) {
		t.Fatalf("expected non-OCC error, got %v", err)
	}
	if mock.queryCalls != 1 {
		t.Fatalf("expected 1 call, got %d", mock.queryCalls)
	}
}

func TestDB_Query_ExhaustsRetries(t *testing.T) {
	mock := &mockPool{
		queryErrs: []error{
			newOCCError("OC000"), newOCCError("OC000"),
			newOCCError("OC000"), newOCCError("OC000"),
		},
	}
	db := New(mock, fastConfig())
	_, err := db.Query(context.Background(), "SELECT * FROM t")
	if err == nil {
		t.Fatal("expected error after exhausting retries")
	}
	if mock.queryCalls != 4 {
		t.Fatalf("expected 4 calls, got %d", mock.queryCalls)
	}
}

func TestDB_Query_NoRetrySkipsRetry(t *testing.T) {
	mock := &mockPool{
		queryErrs: []error{newOCCError("OC000")},
	}
	db := New(mock, fastConfig())
	ctx := NoRetry(context.Background())
	_, err := db.Query(ctx, "SELECT * FROM t")
	if err == nil {
		t.Fatal("expected OCC error to be returned without retry")
	}
	if mock.queryCalls != 1 {
		t.Fatalf("expected 1 call (no retry), got %d", mock.queryCalls)
	}
}

func TestDB_Query_NoRetrySuccess(t *testing.T) {
	mock := &mockPool{}
	db := New(mock, fastConfig())
	ctx := NoRetry(context.Background())
	_, err := db.Query(ctx, "SELECT * FROM t")
	if err != nil {
		t.Fatalf("expected nil error, got %v", err)
	}
	if mock.queryCalls != 1 {
		t.Fatalf("expected 1 call, got %d", mock.queryCalls)
	}
}

// --- QueryRow tests ---

func TestDB_QueryRow_DelegatesDirectly(t *testing.T) {
	mock := &mockPool{}
	db := New(mock, fastConfig())
	db.QueryRow(context.Background(), "SELECT 1")
	if mock.queryRowCalls != 1 {
		t.Fatalf("expected 1 queryRow call, got %d", mock.queryRowCalls)
	}
}

// --- WithTransaction tests ---

func TestDB_WithTransaction_Success(t *testing.T) {
	tx := &mockTx{}
	mock := &mockPool{txSequence: []*mockTx{tx}}
	db := New(mock, fastConfig())

	called := false
	err := db.WithTransaction(context.Background(), func(tx pgx.Tx) error {
		called = true
		return nil
	})
	if err != nil {
		t.Fatalf("expected nil error, got %v", err)
	}
	if !called {
		t.Fatal("expected fn to be called")
	}
	if mock.beginCalls != 1 {
		t.Fatalf("expected 1 begin call, got %d", mock.beginCalls)
	}
	if tx.commitCalls != 1 {
		t.Fatalf("expected 1 commit call, got %d", tx.commitCalls)
	}
}

func TestDB_WithTransaction_RetriesOnOCCAtCommit(t *testing.T) {
	tx1 := &mockTx{commitErr: newOCCError("OC000")}
	tx2 := &mockTx{}
	mock := &mockPool{txSequence: []*mockTx{tx1, tx2}}
	db := New(mock, fastConfig())

	callCount := 0
	err := db.WithTransaction(context.Background(), func(tx pgx.Tx) error {
		callCount++
		return nil
	})
	if err != nil {
		t.Fatalf("expected nil error, got %v", err)
	}
	if callCount != 2 {
		t.Fatalf("expected fn called 2 times, got %d", callCount)
	}
	if mock.beginCalls != 2 {
		t.Fatalf("expected 2 begin calls, got %d", mock.beginCalls)
	}
}

func TestDB_WithTransaction_NoRetrySkipsRetry(t *testing.T) {
	tx := &mockTx{commitErr: newOCCError("OC000")}
	mock := &mockPool{txSequence: []*mockTx{tx}}
	db := New(mock, fastConfig())

	ctx := NoRetry(context.Background())
	err := db.WithTransaction(ctx, func(tx pgx.Tx) error {
		return nil
	})
	if err == nil {
		t.Fatal("expected OCC error without retry")
	}
	if mock.beginCalls != 1 {
		t.Fatalf("expected 1 begin call (no retry), got %d", mock.beginCalls)
	}
}

func TestDB_WithTransaction_FnErrorReturnsImmediately(t *testing.T) {
	tx := &mockTx{}
	mock := &mockPool{txSequence: []*mockTx{tx}}
	db := New(mock, fastConfig())

	fnErr := errors.New("business logic error")
	err := db.WithTransaction(context.Background(), func(tx pgx.Tx) error {
		return fnErr
	})
	if !errors.Is(err, fnErr) {
		t.Fatalf("expected fn error, got %v", err)
	}
	// Should not have committed
	if tx.commitCalls != 0 {
		t.Fatalf("expected 0 commit calls, got %d", tx.commitCalls)
	}
}
