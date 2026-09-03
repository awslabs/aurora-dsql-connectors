/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

package dsql

import (
	"crypto/tls"
	"fmt"
	"net/url"
	"os"
	"strconv"
	"strings"
	"time"

	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/jackc/pgx/v5"
)

// Version is the connector version
const Version = "1.0.0"

// ApplicationName is set on connections for observability
const ApplicationName = "aurora-dsql-go-pgx/" + Version

// Default values for configuration
const (
	DefaultUser     = "admin"
	DefaultDatabase = "postgres"
	DefaultPort     = 5432
)

// Default pool timeouts (matching Node.js connector)
const (
	// DefaultMaxConnLifetime is the default maximum connection lifetime (55 minutes)
	// This aligns with DSQL's connection characteristics
	DefaultMaxConnLifetime = 55 * time.Minute
	// DefaultMaxConnIdleTime is the default maximum idle time (10 minutes)
	DefaultMaxConnIdleTime = 10 * time.Minute
	// DefaultTokenDuration is the default token validity duration (15 minutes)
	DefaultTokenDuration = 15 * time.Minute
)

// Config holds the configuration for connecting to Aurora DSQL.
type Config struct {
	// Host is the cluster endpoint or cluster ID. Required.
	Host string

	// Region is the AWS region. Optional if parseable from Host.
	Region string

	// User is the database user. Default: "admin".
	User string

	// Database is the database name. Default: "postgres".
	Database string

	// Port is the database port. Default: 5432.
	Port int

	// Profile is the AWS profile name for credentials. Optional.
	Profile string

	// TokenDurationSecs is the token validity duration in seconds. Optional.
	TokenDurationSecs int

	// CustomCredentialsProvider is a custom AWS credentials provider. Optional.
	CustomCredentialsProvider aws.CredentialsProvider
}

// resolvedConfig holds the validated and resolved configuration with all
// defaults applied and the full hostname constructed.
type resolvedConfig struct {
	Host                      string
	Region                    string
	User                      string
	Database                  string
	Port                      int
	Profile                   string
	TokenDuration             time.Duration
	CustomCredentialsProvider aws.CredentialsProvider
}

// resolve validates the configuration, applies defaults, and resolves the
// full hostname and region.
func (c *Config) resolve() (*resolvedConfig, error) {
	if c.Host == "" {
		return nil, fmt.Errorf("host is required")
	}

	resolved := &resolvedConfig{
		Host:                      c.Host,
		Region:                    c.Region,
		User:                      c.User,
		Database:                  c.Database,
		Port:                      c.Port,
		Profile:                   c.Profile,
		CustomCredentialsProvider: c.CustomCredentialsProvider,
	}

	// Apply defaults
	if resolved.User == "" {
		resolved.User = DefaultUser
	}
	if resolved.Database == "" {
		resolved.Database = DefaultDatabase
	}
	if resolved.Port == 0 {
		resolved.Port = DefaultPort
	}
	if resolved.Port < 1 || resolved.Port > 65535 {
		return nil, fmt.Errorf("port must be between 1 and 65535, got %d", resolved.Port)
	}

	// Convert token duration with default
	if c.TokenDurationSecs > 0 {
		resolved.TokenDuration = time.Duration(c.TokenDurationSecs) * time.Second
	} else {
		resolved.TokenDuration = DefaultTokenDuration
	}

	// Handle cluster ID vs full hostname
	if IsClusterID(resolved.Host) {
		// Need region to build hostname
		if resolved.Region == "" {
			resolved.Region = getRegionFromEnv()
		}
		if resolved.Region == "" {
			return nil, fmt.Errorf("region is required when host is a cluster ID")
		}
		resolved.Host = BuildHostname(c.Host, resolved.Region)
	} else {
		// Try to parse region from hostname if not provided
		if resolved.Region == "" {
			parsedRegion, err := ParseRegion(resolved.Host)
			if err != nil {
				// Try environment
				resolved.Region = getRegionFromEnv()
				if resolved.Region == "" {
					return nil, fmt.Errorf("region is required: could not parse from hostname and not set in environment")
				}
			} else {
				resolved.Region = parsedRegion
			}
		}
	}

	return resolved, nil
}

// getRegionFromEnv returns the AWS region from environment variables.
func getRegionFromEnv() string {
	if region := os.Getenv("AWS_REGION"); region != "" {
		return region
	}
	return os.Getenv("AWS_DEFAULT_REGION")
}

// ParseConnectionString parses a PostgreSQL or DSQL connection string into a Config.
// Supported schemes: postgres://, postgresql://, dsql://
func ParseConnectionString(connStr string) (*Config, error) {
	// Normalize dsql:// to postgres:// for URL parsing
	normalizedConnStr := connStr
	if strings.HasPrefix(connStr, "dsql://") {
		normalizedConnStr = "postgres://" + strings.TrimPrefix(connStr, "dsql://")
	}

	u, err := url.Parse(normalizedConnStr)
	if err != nil {
		return nil, fmt.Errorf("invalid connection string: %w", err)
	}

	cfg := &Config{
		Host: u.Hostname(),
	}

	if u.User != nil {
		cfg.User = u.User.Username()
	}

	if u.Path != "" && u.Path != "/" {
		cfg.Database = u.Path[1:] // Remove leading slash
	}

	if portStr := u.Port(); portStr != "" {
		port, err := strconv.ParseUint(portStr, 10, 16)
		if err != nil {
			return nil, fmt.Errorf("invalid port: %w", err)
		}
		cfg.Port = int(port)
	}

	// Parse query parameters
	query := u.Query()

	if region := query.Get("region"); region != "" {
		cfg.Region = region
	}

	if profile := query.Get("profile"); profile != "" {
		cfg.Profile = profile
	}

	if tokenDuration := query.Get("tokenDurationSecs"); tokenDuration != "" {
		duration, err := strconv.Atoi(tokenDuration)
		if err != nil {
			return nil, fmt.Errorf("invalid tokenDurationSecs: %w", err)
		}
		cfg.TokenDurationSecs = duration
	}

	return cfg, nil
}

// configureConnConfig sets connection parameters on a pgx.ConnConfig.
func (r *resolvedConfig) configureConnConfig(cfg *pgx.ConnConfig) {
	cfg.Host = r.Host
	cfg.Port = uint16(r.Port)
	cfg.Database = r.Database
	cfg.User = r.User
	cfg.TLSConfig = &tls.Config{
		ServerName: r.Host,
		MinVersion: tls.VersionTLS12,
	}
	cfg.RuntimeParams = map[string]string{
		"application_name": ApplicationName,
	}

	// Use Exec as the default query mode on Aurora DSQL.
	//
	// pgx's default (QueryExecModeCacheDescribe) caches each statement's result
	// column descriptions per connection after the first Describe, then reuses
	// that cached column count to build the Bind result-format-codes array on
	// subsequent executions. Likewise QueryExecModeCacheStatement caches a
	// prepared statement. If a table's schema changes (e.g. ALTER TABLE ADD
	// COLUMN) while pooled connections are still open, those connections hold a
	// stale description/plan and fail on the next execution:
	//
	//	CacheDescribe:  ERROR: bind message has N result formats but query has M columns (SQLSTATE 08P01)
	//	CacheStatement: ERROR: cached plan must not change result type (SQLSTATE 0A000)
	//
	// QueryExecModeExec uses the unnamed prepared statement and does not cache a
	// description or plan across executions, so it always reflects the current
	// schema and neither error can occur. It also completes in a single network
	// round trip. QueryExecModeDescribeExec is likewise schema-change safe but
	// issues a separate Describe round trip on every execution; measured on
	// Aurora DSQL that roughly doubles per-query latency (~2x) versus Exec, with
	// no observed correctness advantage on DSQL's supported type surface (DSQL
	// does not support enums or arrays, which are the main cases where the extra
	// server-side parameter type resolution would matter). Exec is therefore the
	// better default here.
	//
	// Callers may override this after construction, e.g. to DescribeExec if their
	// workload relies on server-driven parameter type resolution.
	cfg.DefaultQueryExecMode = pgx.QueryExecModeExec
}
