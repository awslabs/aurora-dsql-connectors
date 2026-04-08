<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

namespace Aws\AuroraDsql\PdoPgsql\Tests\Integration;

use Aws\AuroraDsql\PdoPgsql\AuroraDsql;
use Aws\AuroraDsql\PdoPgsql\DsqlConfig;
use Aws\AuroraDsql\PdoPgsql\DsqlException;

class ErrorScenarioTest extends IntegrationTestBase
{
    public function testInvalidHostname(): void
    {
        $this->expectException(\PDOException::class);

        $config = new DsqlConfig(
            host: 'invalid-hostname-that-does-not-exist.dsql.us-east-1.on.aws',
            region: 'us-east-1',
            user: 'admin',
            database: 'postgres',
        );

        AuroraDsql::connect($config);
    }

    public function testInvalidDsn(): void
    {
        $this->expectException(DsqlException::class);
        $this->expectExceptionMessage('Invalid connection string');

        AuroraDsql::connectFromDsn('not-a-valid-dsn');
    }

    public function testUnsupportedDsnScheme(): void
    {
        $this->expectException(DsqlException::class);
        $this->expectExceptionMessage('Unsupported scheme');

        AuroraDsql::connectFromDsn('mysql://localhost/test');
    }

    public function testSyntaxErrorThrowsException(): void
    {
        $pdo = $this->createConnection();

        $this->expectException(\PDOException::class);
        $pdo->query('INVALID SQL SYNTAX HERE');
    }

    public function testTableNotFoundError(): void
    {
        $pdo = $this->createConnection();
        $nonExistentTable = 'table_that_does_not_exist_' . bin2hex(random_bytes(8));

        $this->expectException(\PDOException::class);
        $pdo->query("SELECT * FROM {$nonExistentTable}");
    }

    public function testConstraintViolation(): void
    {
        $pdo = $this->createConnection();
        $tableName = $this->generateTableName();

        try {
            // Create table with unique constraint (using UUID like other tests)
            $pdo->exec("
                CREATE TABLE {$tableName} (
                    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
                    unique_value TEXT UNIQUE NOT NULL
                )
            ");

            // Insert first row
            $pdo->exec("INSERT INTO {$tableName} (unique_value) VALUES ('duplicate')");

            // Try to insert duplicate - should fail with unique constraint violation
            $this->expectException(\PDOException::class);
            $pdo->exec("INSERT INTO {$tableName} (unique_value) VALUES ('duplicate')");
        } finally {
            $this->dropTestTable($pdo, $tableName);
        }
    }
}
