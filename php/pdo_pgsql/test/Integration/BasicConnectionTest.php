<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

namespace Aws\AuroraDsql\PdoPgsql\Tests\Integration;

class BasicConnectionTest extends IntegrationTestBase
{
    public function testConnectWithDsqlConfig(): void
    {
        $pdo = $this->createConnection();
        $this->assertInstanceOf(\PDO::class, $pdo);

        // Verify connection is usable
        $result = $pdo->query('SELECT 1 as test')->fetch(\PDO::FETCH_ASSOC);
        $this->assertSame(['test' => '1'], $result);
    }

    public function testConnectWithDsn(): void
    {
        $dsn = sprintf(
            'postgresql://%s/%s?region=%s',
            self::$clusterEndpoint,
            'postgres',
            self::$region ?? 'us-east-1'
        );

        $pdo = \Aws\AuroraDsql\PdoPgsql\AuroraDsql::connectFromDsn($dsn);
        $this->assertInstanceOf(\PDO::class, $pdo);

        // Verify connection is usable
        $result = $pdo->query('SELECT 1 as test')->fetch(\PDO::FETCH_ASSOC);
        $this->assertSame(['test' => '1'], $result);
    }

    public function testConnectionUsesIAMAuthentication(): void
    {
        $pdo = $this->createConnection();

        // Verify we're connected as admin (default IAM user)
        $stmt = $pdo->query('SELECT current_user');
        $user = $stmt->fetchColumn();
        $this->assertSame('admin', $user);
    }

    public function testConnectionAttributesAreSet(): void
    {
        $pdo = $this->createConnection();

        // Verify ERRMODE_EXCEPTION is enforced (required for OCC retry)
        $errMode = $pdo->getAttribute(\PDO::ATTR_ERRMODE);
        $this->assertSame(\PDO::ERRMODE_EXCEPTION, $errMode);
    }
}
