<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

namespace Aws\AuroraDsql\PdoPgsql\Tests\Integration;

/**
 * Statements freed inside a transaction must not discard its writes.
 *
 * Each test verifies the commit from a second connection, because a
 * transaction that the server rolled back still looks committed to the
 * connection that ran it.
 */
class PreparedStatementTransactionTest extends IntegrationTestBase
{
    public function testServerSidePreparesAreDisabledByDefault(): void
    {
        $pdo = $this->createConnection();

        $this->assertTrue($pdo->getAttribute(self::disablePreparesAttribute()));
    }

    public function testExplicitDisablePreparesSettingIsHonored(): void
    {
        $pdo = $this->createConnection(pdoAttributes: [self::disablePreparesAttribute() => false]);

        $this->assertFalse($pdo->getAttribute(self::disablePreparesAttribute()));
    }

    public function testPreparedInsertInTransactionCallbackIsCommitted(): void
    {
        $pdo = $this->createConnection();
        $verifier = $this->createConnection();
        $tableName = $this->generateTableName();

        try {
            $this->createTestTable($pdo, $tableName);

            // $stmt is freed when the callback returns, before transaction() commits
            $id = $pdo->transaction(function (\PDO $conn) use ($tableName): string {
                $stmt = $conn->prepare("INSERT INTO {$tableName} (value, name) VALUES (?, ?) RETURNING id");
                $stmt->execute([1, 'callback']);
                return $stmt->fetchColumn();
            });

            $stmt = $verifier->prepare("SELECT name FROM {$tableName} WHERE id = ?");
            $stmt->execute([$id]);
            $this->assertSame('callback', $stmt->fetchColumn());
        } finally {
            $this->dropTestTable($pdo, $tableName);
        }
    }

    public function testPreparedInsertInManualTransactionIsCommitted(): void
    {
        $pdo = $this->createConnection();
        $verifier = $this->createConnection();
        $tableName = $this->generateTableName();

        try {
            $this->createTestTable($pdo, $tableName);

            $pdo->beginTransaction();
            $pdo->prepare("INSERT INTO {$tableName} (value, name) VALUES (?, ?)")->execute([2, 'manual']);
            $pdo->commit();

            $this->assertSame(1, $this->countRows($verifier, $tableName, 'manual'));
        } finally {
            $this->dropTestTable($pdo, $tableName);
        }
    }

    public function testQueryInManualTransactionKeepsEarlierWrites(): void
    {
        $pdo = $this->createConnection();
        $verifier = $this->createConnection();
        $tableName = $this->generateTableName();

        try {
            $this->createTestTable($pdo, $tableName);

            $pdo->beginTransaction();
            $pdo->exec("INSERT INTO {$tableName} (value, name) VALUES (3, 'query')");
            $this->assertSame(1, (int) $pdo->query("SELECT COUNT(*) FROM {$tableName}")->fetchColumn());
            $pdo->commit();

            $this->assertSame(1, $this->countRows($verifier, $tableName, 'query'));
        } finally {
            $this->dropTestTable($pdo, $tableName);
        }
    }

    private function countRows(\PDO $pdo, string $tableName, string $name): int
    {
        $stmt = $pdo->prepare("SELECT COUNT(*) FROM {$tableName} WHERE name = ?");
        $stmt->execute([$name]);
        return (int) $stmt->fetchColumn();
    }

    private static function disablePreparesAttribute(): int
    {
        return class_exists(\Pdo\Pgsql::class, false)
            ? \Pdo\Pgsql::ATTR_DISABLE_PREPARES
            : \PDO::PGSQL_ATTR_DISABLE_PREPARES;
    }
}
