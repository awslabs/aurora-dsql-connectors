<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

namespace Aws\AuroraDsql\PdoPgsql\Tests\Integration;

class DsnParsingTest extends IntegrationTestBase
{
    public function testConnectWithMinimalDsn(): void
    {
        $dsn = sprintf(
            'postgresql://%s',
            self::$clusterEndpoint
        );

        $pdo = \Aws\AuroraDsql\PdoPgsql\AuroraDsql::connectFromDsn($dsn);
        $this->assertInstanceOf(\PDO::class, $pdo);

        // Verify connection with defaults (admin user, postgres database)
        $result = $pdo->query('SELECT current_database()')->fetchColumn();
        $this->assertSame('postgres', $result);
    }

    public function testConnectWithRegionParameter(): void
    {
        $dsn = sprintf(
            'postgresql://%s/postgres?region=%s',
            self::$clusterEndpoint,
            self::$region ?? 'us-east-1'
        );

        $pdo = \Aws\AuroraDsql\PdoPgsql\AuroraDsql::connectFromDsn($dsn);
        $this->assertInstanceOf(\PDO::class, $pdo);

        $result = $pdo->query('SELECT 1')->fetchColumn();
        $this->assertSame('1', $result);
    }

    public function testConnectWithCustomPort(): void
    {
        $dsn = sprintf(
            'postgresql://%s:5432/postgres?region=%s',
            self::$clusterEndpoint,
            self::$region ?? 'us-east-1'
        );

        $pdo = \Aws\AuroraDsql\PdoPgsql\AuroraDsql::connectFromDsn($dsn);
        $this->assertInstanceOf(\PDO::class, $pdo);

        $result = $pdo->query('SELECT 1')->fetchColumn();
        $this->assertSame('1', $result);
    }
}
