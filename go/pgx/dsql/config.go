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

// DefaultQueryExecMode is the pgx query execution mode the connector applies
// when Config.QueryExecMode is left unset. See configureConnConfig for the
// rationale.
const DefaultQueryExecMode = pgx.QueryExecModeDescribeExec

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

	// QueryExecMode overrides the pgx query execution mode. Optional; zero means
	// the connector default (QueryExecModeDescribeExec). See configureConnConfig.
	QueryExecMode pgx.QueryExecMode
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
	QueryExecMode             pgx.QueryExecMode
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
		QueryExecMode:             c.QueryExecMode,
	}

	// Apply defaults
	if resolved.User == "" {
		resolved.User = DefaultUser
	}
	// Zero is not a valid pgx.QueryExecMode, so treat it as "unset".
	if resolved.QueryExecMode == 0 {
		resolved.QueryExecMode = DefaultQueryExecMode
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

func parseQueryExecMode(value string) (pgx.QueryExecMode, error) {
	switch value {
	case "cache_statement":
		return pgx.QueryExecModeCacheStatement, nil
	case "cache_describe":
		return pgx.QueryExecModeCacheDescribe, nil
	case "describe_exec":
		return pgx.QueryExecModeDescribeExec, nil
	case "exec":
		return pgx.QueryExecModeExec, nil
	case "simple_protocol":
		return pgx.QueryExecModeSimpleProtocol, nil
	default:
		return 0, fmt.Errorf("invalid queryExecMode %q", value)
	}
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

	if queryExecMode := query.Get("queryExecMode"); queryExecMode != "" {
		mode, err := parseQueryExecMode(queryExecMode)
		if err != nil {
			return nil, err
		}
		cfg.QueryExecMode = mode
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

	// Default to DescribeExec (overridable via Config.QueryExecMode). pgx's
	// caching modes (CacheStatement, CacheDescribe) fail after a live schema
	// change on connections holding a stale plan/description (0A000 / 08P01).
	// DescribeExec re-Describes each execution, so it is schema-change safe and
	// resolves parameter types server-side. Exec avoids the extra round trip but
	// infers param types from Go types alone, which on DSQL rejects jsonb
	// map/[]byte params and silently corrupts []byte into text columns, so it is
	// opt-in only.
	cfg.DefaultQueryExecMode = r.QueryExecMode
}
