/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

package dsql

import (
	"context"
	"fmt"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
)

// NewPool creates a new connection pool to Aurora DSQL.
//
// The config parameter can be a Config struct, *Config, or a connection string.
//
// The optional poolConfig parameter allows direct configuration of the underlying pgxpool.
// It must be created via [pgxpool.ParseConfig]. If omitted, sensible defaults are applied
// (MaxConnLifetime: 55min, MaxConnIdleTime: 10min). Any BeforeConnect callback set on
// poolConfig will be chained with the connector's IAM token generation (user callback
// runs first).
func NewPool(ctx context.Context, config any, poolConfig ...*pgxpool.Config) (*pgxpool.Pool, error) {
	var cfg *Config

	switch c := config.(type) {
	case Config:
		cfg = &c
	case *Config:
		if c == nil {
			return nil, fmt.Errorf("config cannot be nil")
		}
		cfg = c
	case string:
		parsed, err := ParseConnectionString(c)
		if err != nil {
			return nil, err
		}
		cfg = parsed
	default:
		return nil, fmt.Errorf("config must be Config, *Config, or string, got %T", config)
	}

	resolved, err := cfg.resolve()
	if err != nil {
		return nil, err
	}

	var pc *pgxpool.Config
	if len(poolConfig) > 0 {
		pc = poolConfig[0]
	}

	return newPoolFromResolved(ctx, resolved, pc)
}

func newPoolFromResolved(ctx context.Context, resolved *resolvedConfig, poolConfig *pgxpool.Config) (*pgxpool.Pool, error) {
	credentialsProvider, err := resolveCredentialsProvider(ctx, resolved)
	if err != nil {
		return nil, fmt.Errorf("failed to resolve credentials provider: %w", err)
	}

	applyDSQLDefaults := poolConfig == nil
	if poolConfig == nil {
		poolConfig, err = pgxpool.ParseConfig("")
		if err != nil {
			return nil, fmt.Errorf("unable to create pool config: %w", err)
		}
	}

	resolved.configureConnConfig(poolConfig.ConnConfig)

	// Apply DSQL-optimized defaults. When no pool config was provided,
	// always override pgxpool defaults. When the user provides their own
	// config, only fill in zero values so that users who don't explicitly
	// set lifetimes still get safe connection recycling on DSQL (where
	// connections timeout server-side after 60 minutes).
	if applyDSQLDefaults || poolConfig.MaxConnLifetime == 0 {
		poolConfig.MaxConnLifetime = DefaultMaxConnLifetime
	}
	if applyDSQLDefaults || poolConfig.MaxConnIdleTime == 0 {
		poolConfig.MaxConnIdleTime = DefaultMaxConnIdleTime
	}

	// Chain with any user-provided BeforeConnect callback
	userBeforeConnect := poolConfig.BeforeConnect
	poolConfig.BeforeConnect = func(ctx context.Context, cfg *pgx.ConnConfig) error {
		if userBeforeConnect != nil {
			if err := userBeforeConnect(ctx, cfg); err != nil {
				return err
			}
		}
		token, err := GenerateToken(ctx, resolved.Host, resolved.Region, resolved.User, credentialsProvider, resolved.TokenDuration)
		if err != nil {
			return err
		}
		cfg.Password = token
		return nil
	}

	pool, err := pgxpool.NewWithConfig(ctx, poolConfig)
	if err != nil {
		return nil, fmt.Errorf("unable to create connection pool: %w", err)
	}

	return pool, nil
}
