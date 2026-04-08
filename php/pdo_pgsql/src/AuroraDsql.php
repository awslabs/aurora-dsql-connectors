<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

namespace Aws\AuroraDsql\PdoPgsql;

/**
 * Factory for creating Aurora DSQL PDO connections with automatic IAM authentication.
 *
 * Provides static factory methods to create DsqlPdo instances with:
 * - Automatic IAM token generation using AWS credentials
 * - SSL enforcement (sslmode=verify-full)
 * - Optional OCC retry with exponential backoff
 *
 * Example:
 * ```php
 * $config = new DsqlConfig(host: 'cluster.dsql.us-east-1.on.aws');
 * $pdo = AuroraDsql::connect($config);
 * ```
 *
 * @see DsqlConfig
 * @see DsqlPdo
 */
class AuroraDsql
{
    public static function connect(DsqlConfig $config, array $pdoAttributes = []): DsqlPdo
    {
        $resolved = $config->resolve();
        return self::createPdo($resolved, $pdoAttributes);
    }

    public static function connectFromDsn(string $dsn, array $pdoAttributes = []): DsqlPdo
    {
        $config = DsqlConfig::parse($dsn);
        return self::connect($config, $pdoAttributes);
    }

    private static function createPdo(ResolvedConfig $resolved, array $pdoAttributes): DsqlPdo
    {
        $token = Token::generate(
            host: $resolved->host,
            region: $resolved->region,
            user: $resolved->user,
            expiresInSecs: $resolved->tokenDurationSecs,
            credentialsProvider: $resolved->credentialsProvider,
            profile: $resolved->profile,
        );

        $dsn = $resolved->toDsn();

        // Inject sslnegotiation=direct for libpq 17+
        if (self::supportsDirectSslNegotiation()) {
            $dsn .= ';sslnegotiation=direct';
        }

        // Force ERRMODE_EXCEPTION — OCC retry depends on exceptions being thrown
        $mergedAttributes = $pdoAttributes;
        $mergedAttributes[\PDO::ATTR_ERRMODE] = \PDO::ERRMODE_EXCEPTION;

        return new DsqlPdo(
            dsn: $dsn,
            username: $resolved->user,
            password: $token,
            options: $mergedAttributes,
            occMaxRetries: $resolved->occMaxRetries ?? 0,
            logger: $resolved->logger,
        );
    }

    private static function supportsDirectSslNegotiation(): bool
    {
        // Check libpq version via PGSQL_LIBPQ_VERSION constant (ext-pgsql)
        if (defined('PGSQL_LIBPQ_VERSION')) {
            return version_compare(PGSQL_LIBPQ_VERSION, '17.0', '>=');
        }

        // If ext-pgsql isn't loaded (only ext-pdo_pgsql), skip sslnegotiation=direct.
        // This is safe — it just means an extra round trip during SSL negotiation.
        return false;
    }
}
