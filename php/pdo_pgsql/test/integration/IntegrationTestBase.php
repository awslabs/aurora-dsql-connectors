<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

namespace Aws\AuroraDsql\PdoPgsql\Tests\Integration;

use Aws\AuroraDsql\PdoPgsql\AuroraDsql;
use Aws\AuroraDsql\PdoPgsql\DsqlConfig;
use PHPUnit\Framework\TestCase;

abstract class IntegrationTestBase extends TestCase
{
    protected static string $clusterEndpoint;
    protected static ?string $region;

    public static function setUpBeforeClass(): void
    {
        self::$clusterEndpoint = getenv('CLUSTER_ENDPOINT');
        if (!self::$clusterEndpoint) {
            throw new \RuntimeException(
                'CLUSTER_ENDPOINT environment variable is required for integration tests. ' .
                'Set it to your Aurora DSQL cluster endpoint.'
            );
        }

        self::$region = getenv('REGION') ?: null;
    }

    protected function createConnection(array $configOverrides = []): \PDO
    {
        $config = new DsqlConfig(
            host: $configOverrides['host'] ?? self::$clusterEndpoint,
            region: $configOverrides['region'] ?? self::$region,
            user: $configOverrides['user'] ?? 'admin',
            database: $configOverrides['database'] ?? 'postgres',
            port: $configOverrides['port'] ?? 5432,
            profile: $configOverrides['profile'] ?? null,
            credentialsProvider: $configOverrides['credentialsProvider'] ?? null,
            tokenDurationSecs: $configOverrides['tokenDurationSecs'] ?? 900,
            ormPrefix: $configOverrides['ormPrefix'] ?? null,
        );

        return AuroraDsql::connect($config);
    }

    protected function generateTableName(string $prefix = 'test'): string
    {
        return sprintf('%s_%s_%d', $prefix, bin2hex(random_bytes(4)), getmypid());
    }

    protected function createTestTable(\PDO $pdo, string $tableName): void
    {
        $pdo->exec(sprintf(
            'CREATE TABLE IF NOT EXISTS %s (
                id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
                value INT NOT NULL DEFAULT 0,
                name TEXT
            )',
            $tableName
        ));
    }

    protected function dropTestTable(\PDO $pdo, string $tableName): void
    {
        try {
            $pdo->exec(sprintf('DROP TABLE IF EXISTS %s', $tableName));
        } catch (\PDOException $e) {
            // Ignore cleanup errors
        }
    }
}
