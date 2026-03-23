/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

package dsql

import (
	"context"
	"os"
	"testing"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNewPoolNilConfig(t *testing.T) {
	ctx := context.Background()
	var nilConfig *Config = nil
	_, err := NewPool(ctx, nilConfig)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "config cannot be nil")
}

func TestNewPool(t *testing.T) {
	endpoint := os.Getenv("CLUSTER_ENDPOINT")
	region := os.Getenv("REGION")
	if endpoint == "" || region == "" {
		t.Skip("CLUSTER_ENDPOINT and REGION required for pool test")
	}

	ctx := context.Background()

	pool, err := NewPool(ctx, Config{
		Host:   endpoint,
		Region: region,
	})
	require.NoError(t, err)
	defer pool.Close()

	// Verify connection works
	err = pool.Ping(ctx)
	assert.NoError(t, err)
}

func TestNewPoolFromConnectionString(t *testing.T) {
	endpoint := os.Getenv("CLUSTER_ENDPOINT")
	region := os.Getenv("REGION")
	if endpoint == "" || region == "" {
		t.Skip("CLUSTER_ENDPOINT and REGION required for pool test")
	}

	ctx := context.Background()
	connStr := "postgres://admin@" + endpoint + "/postgres?region=" + region

	pool, err := NewPool(ctx, connStr)
	require.NoError(t, err)
	defer pool.Close()

	// Verify connection works
	err = pool.Ping(ctx)
	assert.NoError(t, err)
}

func TestPoolQuery(t *testing.T) {
	endpoint := os.Getenv("CLUSTER_ENDPOINT")
	region := os.Getenv("REGION")
	if endpoint == "" || region == "" {
		t.Skip("CLUSTER_ENDPOINT and REGION required for pool test")
	}

	ctx := context.Background()

	pool, err := NewPool(ctx, Config{
		Host:   endpoint,
		Region: region,
	})
	require.NoError(t, err)
	defer pool.Close()

	var result int
	err = pool.QueryRow(ctx, "SELECT 1").Scan(&result)
	require.NoError(t, err)
	assert.Equal(t, 1, result)
}

func TestPoolWithCustomPoolConfig(t *testing.T) {
	endpoint := os.Getenv("CLUSTER_ENDPOINT")
	region := os.Getenv("REGION")
	if endpoint == "" || region == "" {
		t.Skip("CLUSTER_ENDPOINT and REGION required for pool test")
	}

	ctx := context.Background()

	poolCfg, err := pgxpool.ParseConfig("")
	require.NoError(t, err)
	poolCfg.MaxConns = 5
	poolCfg.MinConns = 1

	pool, err := NewPool(ctx, Config{
		Host:   endpoint,
		Region: region,
	}, poolCfg)
	require.NoError(t, err)
	defer pool.Close()

	// Verify pool config
	actualCfg := pool.Config()
	assert.Equal(t, int32(5), actualCfg.MaxConns)
	assert.Equal(t, int32(1), actualCfg.MinConns)
}

func TestPoolDefaultsApplied(t *testing.T) {
	endpoint := os.Getenv("CLUSTER_ENDPOINT")
	region := os.Getenv("REGION")
	if endpoint == "" || region == "" {
		t.Skip("CLUSTER_ENDPOINT and REGION required for pool test")
	}

	ctx := context.Background()

	pool, err := NewPool(ctx, Config{
		Host:   endpoint,
		Region: region,
	})
	require.NoError(t, err)
	defer pool.Close()

	// Verify DSQL defaults are applied
	actualCfg := pool.Config()
	assert.Equal(t, 55*time.Minute, actualCfg.MaxConnLifetime)
	assert.Equal(t, 10*time.Minute, actualCfg.MaxConnIdleTime)
}

func TestPoolUserConfigPreserved(t *testing.T) {
	endpoint := os.Getenv("CLUSTER_ENDPOINT")
	region := os.Getenv("REGION")
	if endpoint == "" || region == "" {
		t.Skip("CLUSTER_ENDPOINT and REGION required for pool test")
	}

	ctx := context.Background()

	poolCfg, err := pgxpool.ParseConfig("")
	require.NoError(t, err)
	poolCfg.MaxConnLifetime = 30 * time.Minute
	poolCfg.MaxConnIdleTime = 5 * time.Minute
	poolCfg.MaxConnLifetimeJitter = 3 * time.Minute

	pool, err := NewPool(ctx, Config{
		Host:   endpoint,
		Region: region,
	}, poolCfg)
	require.NoError(t, err)
	defer pool.Close()

	// Verify user-provided values are preserved, not overwritten by defaults
	actualCfg := pool.Config()
	assert.Equal(t, 30*time.Minute, actualCfg.MaxConnLifetime)
	assert.Equal(t, 5*time.Minute, actualCfg.MaxConnIdleTime)
	assert.Equal(t, 3*time.Minute, actualCfg.MaxConnLifetimeJitter)
}
