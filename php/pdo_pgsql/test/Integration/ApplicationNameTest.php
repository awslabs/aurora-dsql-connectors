<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

namespace Aws\AuroraDsql\PdoPgsql\Tests\Integration;

use Aws\AuroraDsql\PdoPgsql\AuroraDsql;
use Aws\AuroraDsql\PdoPgsql\DsqlConfig;

class ApplicationNameTest extends IntegrationTestBase
{
    public function testDefaultApplicationName(): void
    {
        $pdo = $this->createConnection();

        // Query application_name from pg_stat_activity
        $stmt = $pdo->query(
            "SELECT application_name FROM pg_stat_activity WHERE pid = pg_backend_pid()"
        );
        $appName = $stmt->fetchColumn();

        $this->assertNotEmpty($appName);
        $this->assertStringContainsString('aurora-dsql-php-pdo-pgsql', $appName);
    }

    public function testApplicationNameWithOrmPrefix(): void
    {
        $config = new DsqlConfig(
            host: self::$clusterEndpoint,
            region: self::$region,
            user: 'admin',
            database: 'postgres',
            port: 5432,
            ormPrefix: 'Laravel',
        );

        $pdo = AuroraDsql::connect($config);

        // Query application_name
        $stmt = $pdo->query(
            "SELECT application_name FROM pg_stat_activity WHERE pid = pg_backend_pid()"
        );
        $appName = $stmt->fetchColumn();

        $this->assertNotEmpty($appName);
        $this->assertStringStartsWith('Laravel:', $appName);
        $this->assertStringContainsString('aurora-dsql-php-pdo-pgsql', $appName);
    }

    public function testApplicationNameInDsn(): void
    {
        $dsn = sprintf(
            'postgresql://%s/postgres?region=%s&ormPrefix=Symfony',
            self::$clusterEndpoint,
            self::$region ?? 'us-east-1'
        );

        $pdo = AuroraDsql::connectFromDsn($dsn);

        // Query application_name
        $stmt = $pdo->query(
            "SELECT application_name FROM pg_stat_activity WHERE pid = pg_backend_pid()"
        );
        $appName = $stmt->fetchColumn();

        $this->assertNotEmpty($appName);
        $this->assertStringStartsWith('Symfony:', $appName);
        $this->assertStringContainsString('aurora-dsql-php-pdo-pgsql', $appName);
    }
}
