<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

namespace Aws\AuroraDsql\PdoPgsql;

use Psr\Log\LoggerInterface;

/**
 * Configuration for Aurora DSQL connections.
 *
 * The host parameter accepts either:
 * - Full DSQL hostname (e.g., 'a1b2c3d4e5f6g7h8i9j0klmnop.dsql.us-east-1.on.aws')
 * - 26-character cluster ID (lowercase alphanumeric, e.g., 'a1b2c3d4e5f6g7h8i9j0klmnop')
 *
 * When using a cluster ID, you must also provide the region parameter.
 * AWS region can be auto-detected from full hostnames.
 *
 * Example with full hostname:
 * ```php
 * $config = new DsqlConfig(host: 'a1b2c3d4e5f6g7h8i9j0klmnop.dsql.us-east-1.on.aws');
 * ```
 *
 * Example with cluster ID:
 * ```php
 * $config = new DsqlConfig(
 *     host: 'a1b2c3d4e5f6g7h8i9j0klmnop',
 *     region: 'us-east-1'
 * );
 * ```
 *
 * @see AuroraDsql::connect()
 * @see DsqlConfig::parse() For parsing connection strings
 */
class DsqlConfig
{
    public function __construct(
        public readonly string $host,
        public readonly ?string $region = null,
        public readonly string $user = 'admin',
        public readonly string $database = 'postgres',
        public readonly int $port = 5432,
        public readonly ?string $profile = null,
        public readonly ?\Closure $credentialsProvider = null,
        public readonly int $tokenDurationSecs = 900,
        public readonly ?string $ormPrefix = null,
        public readonly ?int $occMaxRetries = null,
        public readonly ?LoggerInterface $logger = null,
    ) {
    }

    /** @internal */
    public function resolve(): ResolvedConfig
    {
        $host = $this->host;
        $region = $this->region;

        // Expand cluster ID to full hostname
        if (Util::isClusterId($host)) {
            if ($region === null) {
                $region = Util::regionFromEnv();
            }
            if ($region === null) {
                throw new DsqlException(
                    'Cannot resolve cluster ID to hostname: region is required '
                    . 'when host is a cluster ID. Set region explicitly or via '
                    . 'AWS_REGION / AWS_DEFAULT_REGION environment variables.'
                );
            }
            $host = Util::buildHostname($host, $region);
        }

        // Parse region from hostname if not explicitly provided
        if ($region === null) {
            $region = Util::parseRegion($host);
        }
        if ($region === null) {
            $region = Util::regionFromEnv();
        }
        if ($region === null) {
            throw new DsqlException(
                'Cannot determine AWS region. Set region explicitly, use a DSQL '
                . 'hostname (*.dsql.<region>.on.aws), or set AWS_REGION / '
                . 'AWS_DEFAULT_REGION environment variables.'
            );
        }

        if ($this->occMaxRetries !== null && $this->occMaxRetries < 0) {
            throw new DsqlException(
                "occMaxRetries must be null, 0, or positive, got {$this->occMaxRetries}"
            );
        }

        return new ResolvedConfig(
            host: $host,
            region: $region,
            user: $this->user,
            database: $this->database,
            port: $this->port,
            profile: $this->profile,
            credentialsProvider: $this->credentialsProvider,
            tokenDurationSecs: $this->tokenDurationSecs,
            applicationName: Version::buildApplicationName($this->ormPrefix),
            occMaxRetries: $this->occMaxRetries,
            logger: $this->logger,
        );
    }

    public static function parse(string $dsn): self
    {
        $parts = parse_url($dsn);
        if ($parts === false || !isset($parts['host'])) {
            throw new DsqlException("Invalid connection string: {$dsn}");
        }

        $scheme = $parts['scheme'] ?? '';
        if (!in_array($scheme, ['postgres', 'postgresql'], true)) {
            throw new DsqlException(
                "Unsupported scheme '{$scheme}'. Use postgres:// or postgresql://"
            );
        }

        // Parse query params
        $queryParams = [];
        if (isset($parts['query'])) {
            parse_str($parts['query'], $queryParams);
        }

        // Validate query params — reject unrecognized parameters early
        $knownParams = ['region', 'profile', 'tokenDurationSecs', 'ormPrefix', 'occMaxRetries'];
        $unrecognized = array_diff(array_keys($queryParams), $knownParams);
        if ($unrecognized !== []) {
            throw new DsqlException(sprintf(
                'Unrecognized connection string parameter(s): %s. Valid parameters are: %s',
                implode(', ', $unrecognized),
                implode(', ', $knownParams),
            ));
        }

        // Extract DSQL-specific params
        $region = $queryParams['region'] ?? null;
        $profile = $queryParams['profile'] ?? null;
        $tokenDurationSecs = isset($queryParams['tokenDurationSecs'])
            ? (int) $queryParams['tokenDurationSecs']
            : 900;

        $host = $parts['host'];
        $user = isset($parts['user']) && $parts['user'] !== '' ? $parts['user'] : 'admin';
        $port = $parts['port'] ?? 5432;
        $database = isset($parts['path']) && $parts['path'] !== '/'
            ? ltrim($parts['path'], '/')
            : 'postgres';

        $ormPrefix = $queryParams['ormPrefix'] ?? null;
        $occMaxRetries = isset($queryParams['occMaxRetries'])
            ? (int) $queryParams['occMaxRetries']
            : null;

        return new self(
            host: $host,
            region: $region,
            user: $user,
            database: $database,
            port: $port,
            profile: $profile,
            tokenDurationSecs: $tokenDurationSecs,
            ormPrefix: $ormPrefix,
            occMaxRetries: $occMaxRetries,
        );
    }
}
