<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

namespace Aws\AuroraDsql\PdoPgsql\Tests\Integration;

class DatabaseOperationsTest extends IntegrationTestBase
{
    public function testCreateTable(): void
    {
        $pdo = $this->createConnection();
        $tableName = $this->generateTableName();

        $this->createTestTable($pdo, $tableName);

        // Verify table exists by querying information_schema
        $stmt = $pdo->prepare(
            "SELECT table_name FROM information_schema.tables
             WHERE table_schema = 'public' AND table_name = ?"
        );
        $stmt->execute([$tableName]);
        $result = $stmt->fetchColumn();

        $this->assertSame($tableName, $result);

        $this->dropTestTable($pdo, $tableName);
    }

    public function testInsertAndSelect(): void
    {
        $pdo = $this->createConnection();
        $tableName = $this->generateTableName();

        try {
            $this->createTestTable($pdo, $tableName);

            // Insert a row
            $stmt = $pdo->prepare("INSERT INTO {$tableName} (value, name) VALUES (?, ?) RETURNING id");
            $stmt->execute([42, 'test_value']);
            $id = $stmt->fetchColumn();

            $this->assertNotEmpty($id);

            // Select the row back
            $stmt = $pdo->prepare("SELECT value, name FROM {$tableName} WHERE id = ?");
            $stmt->execute([$id]);
            $row = $stmt->fetch(\PDO::FETCH_ASSOC);

            $this->assertEquals(42, $row['value']);
            $this->assertSame('test_value', $row['name']);
        } finally {
            $this->dropTestTable($pdo, $tableName);
        }
    }

    public function testUpdateAndDelete(): void
    {
        $pdo = $this->createConnection();
        $tableName = $this->generateTableName();

        try {
            $this->createTestTable($pdo, $tableName);

            // Insert a row
            $stmt = $pdo->prepare("INSERT INTO {$tableName} (value, name) VALUES (?, ?) RETURNING id");
            $stmt->execute([10, 'original']);
            $id = $stmt->fetchColumn();

            // Update the row
            $stmt = $pdo->prepare("UPDATE {$tableName} SET value = ?, name = ? WHERE id = ?");
            $stmt->execute([20, 'updated', $id]);

            // Verify update
            $stmt = $pdo->prepare("SELECT value, name FROM {$tableName} WHERE id = ?");
            $stmt->execute([$id]);
            $row = $stmt->fetch(\PDO::FETCH_ASSOC);
            $this->assertEquals(20, $row['value']);
            $this->assertSame('updated', $row['name']);

            // Delete the row
            $stmt = $pdo->prepare("DELETE FROM {$tableName} WHERE id = ?");
            $stmt->execute([$id]);

            // Verify deletion
            $stmt = $pdo->prepare("SELECT COUNT(*) FROM {$tableName} WHERE id = ?");
            $stmt->execute([$id]);
            $count = $stmt->fetchColumn();
            $this->assertEquals('0', $count);
        } finally {
            $this->dropTestTable($pdo, $tableName);
        }
    }

    public function testTransactionCommit(): void
    {
        $pdo = $this->createConnection();
        $tableName = $this->generateTableName();

        try {
            $this->createTestTable($pdo, $tableName);

            $pdo->beginTransaction();
            $stmt = $pdo->prepare("INSERT INTO {$tableName} (value, name) VALUES (?, ?)");
            $stmt->execute([100, 'committed']);
            $pdo->commit();

            // Verify data was committed
            $stmt = $pdo->prepare("SELECT COUNT(*) FROM {$tableName} WHERE value = ?");
            $stmt->execute([100]);
            $count = $stmt->fetchColumn();
            $this->assertEquals('1', $count);
        } finally {
            $this->dropTestTable($pdo, $tableName);
        }
    }

    public function testTransactionRollback(): void
    {
        $pdo = $this->createConnection();
        $tableName = $this->generateTableName();

        try {
            $this->createTestTable($pdo, $tableName);

            $pdo->beginTransaction();
            $stmt = $pdo->prepare("INSERT INTO {$tableName} (value, name) VALUES (?, ?)");
            $stmt->execute([200, 'rollback']);
            $pdo->rollBack();

            // Verify data was rolled back
            $stmt = $pdo->prepare("SELECT COUNT(*) FROM {$tableName} WHERE value = ?");
            $stmt->execute([200]);
            $count = $stmt->fetchColumn();
            $this->assertEquals('0', $count);
        } finally {
            $this->dropTestTable($pdo, $tableName);
        }
    }
}
