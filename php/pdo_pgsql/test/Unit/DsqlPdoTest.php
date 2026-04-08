<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

namespace Aws\AuroraDsql\PdoPgsql\Tests\Unit;

use Aws\AuroraDsql\PdoPgsql\DsqlException;
use Aws\AuroraDsql\PdoPgsql\DsqlPdo;
use PHPUnit\Framework\TestCase;
use Psr\Log\LoggerInterface;

class DsqlPdoTest extends TestCase
{
    private function createSqlitePdo(int $occMaxRetries = 0, ?LoggerInterface $logger = null): DsqlPdo
    {
        return new DsqlPdo(
            dsn: 'sqlite::memory:',
            occMaxRetries: $occMaxRetries,
            logger: $logger,
        );
    }

    // --- transaction() success path ---

    public function testTransactionCommitsOnSuccess(): void
    {
        $pdo = $this->createSqlitePdo();
        $pdo->exec('CREATE TABLE t (id INTEGER PRIMARY KEY, val TEXT)');

        $result = $pdo->transaction(function (\PDO $conn): string {
            $conn->exec("INSERT INTO t (val) VALUES ('hello')");
            return 'done';
        });

        $this->assertSame('done', $result);

        $stmt = $pdo->query('SELECT val FROM t');
        $this->assertSame('hello', $stmt->fetchColumn());
    }

    public function testTransactionRollsBackOnNonOccError(): void
    {
        $pdo = $this->createSqlitePdo();
        $pdo->exec('CREATE TABLE t (id INTEGER PRIMARY KEY, val TEXT)');

        try {
            $pdo->transaction(function (\PDO $conn): void {
                $conn->exec("INSERT INTO t (val) VALUES ('should-rollback')");
                throw new \RuntimeException('app error');
            });
            $this->fail('Expected RuntimeException');
        } catch (\RuntimeException $e) {
            $this->assertSame('app error', $e->getMessage());
        }

        $stmt = $pdo->query('SELECT COUNT(*) FROM t');
        $this->assertSame(0, (int) $stmt->fetchColumn());
    }

    // --- OCC retry ---

    public function testTransactionRetriesOnOccError(): void
    {
        $pdo = $this->createSqlitePdo(occMaxRetries: 3);
        $pdo->exec('CREATE TABLE t (id INTEGER PRIMARY KEY, val TEXT)');

        $attempt = 0;
        $result = $pdo->transaction(function (\PDO $conn) use (&$attempt): string {
            $attempt++;
            if ($attempt <= 2) {
                throw self::makeOccException();
            }
            $conn->exec("INSERT INTO t (val) VALUES ('retried')");
            return 'ok';
        });

        $this->assertSame(3, $attempt);
        $this->assertSame('ok', $result);
    }

    public function testTransactionThrowsDsqlExceptionAfterMaxRetries(): void
    {
        $pdo = $this->createSqlitePdo(occMaxRetries: 2);

        try {
            $pdo->transaction(function (\PDO $conn): void {
                throw self::makeOccException();
            });
            $this->fail('Expected DsqlException');
        } catch (DsqlException $e) {
            $this->assertStringContainsString('OCC max retries (2) exceeded', $e->getMessage());
            $this->assertInstanceOf(\PDOException::class, $e->getPrevious());
        }
    }

    public function testTransactionNoRetryWhenOccMaxRetriesIsZero(): void
    {
        $pdo = $this->createSqlitePdo(occMaxRetries: 0);

        $attempt = 0;
        try {
            $pdo->transaction(function (\PDO $conn) use (&$attempt): void {
                $attempt++;
                throw self::makeOccException();
            });
        } catch (\PDOException $e) {
            // With 0 retries, OCC error is thrown directly (not wrapped in DsqlException)
        }

        $this->assertSame(1, $attempt);
    }

    public function testTransactionPerCallOverride(): void
    {
        $pdo = $this->createSqlitePdo(occMaxRetries: 0); // default: no retry

        $attempt = 0;
        $pdo->transaction(function (\PDO $conn) use (&$attempt): void {
            $attempt++;
            if ($attempt === 1) {
                throw self::makeOccException();
            }
        }, maxRetries: 2); // per-call override

        $this->assertSame(2, $attempt);
    }

    public function testTransactionRethrowsNonOccErrorImmediately(): void
    {
        $pdo = $this->createSqlitePdo(occMaxRetries: 3);

        $attempt = 0;
        try {
            $pdo->transaction(function (\PDO $conn) use (&$attempt): void {
                $attempt++;
                throw new \RuntimeException('not an OCC error');
            });
        } catch (\RuntimeException $e) {
            $this->assertSame('not an OCC error', $e->getMessage());
        }

        $this->assertSame(1, $attempt, 'Should not retry non-OCC errors');
    }

    // --- exec() transparent retry ---

    public function testExecWorksNormally(): void
    {
        $pdo = $this->createSqlitePdo(occMaxRetries: 3);
        $pdo->exec('CREATE TABLE t (id INTEGER PRIMARY KEY, val TEXT)');
        $pdo->exec("INSERT INTO t (val) VALUES ('hello')");

        $stmt = $pdo->query('SELECT val FROM t');
        $this->assertSame('hello', $stmt->fetchColumn());
    }

    public function testExecNoRetryWhenOccMaxRetriesIsZero(): void
    {
        $pdo = $this->createSqlitePdo(occMaxRetries: 0);
        $pdo->exec('CREATE TABLE t (id INTEGER PRIMARY KEY, val TEXT)');

        // Normal exec should work fine with 0 retries
        $affected = $pdo->exec("INSERT INTO t (val) VALUES ('test')");
        $this->assertSame(1, $affected);
    }

    public function testExecInsideTransactionDelegatesToParent(): void
    {
        // When inside a transaction, exec() should NOT independently retry —
        // the enclosing transaction() handles retry at the transaction level.
        $pdo = $this->createSqlitePdo(occMaxRetries: 3);
        $pdo->exec('CREATE TABLE t (id INTEGER PRIMARY KEY, val TEXT)');

        $pdo->transaction(function (\PDO $conn): void {
            // This exec() is inside a transaction, so it goes through parent::exec()
            $conn->exec("INSERT INTO t (val) VALUES ('inside-tx')");
        });

        $stmt = $pdo->query('SELECT val FROM t');
        $this->assertSame('inside-tx', $stmt->fetchColumn());
    }

    // --- logging ---

    public function testTransactionLogsOnRetry(): void
    {
        $logger = $this->createMock(LoggerInterface::class);
        $logger->expects($this->once())
            ->method('warning')
            ->with($this->stringContains('OCC conflict detected, retrying'));

        $pdo = $this->createSqlitePdo(occMaxRetries: 3, logger: $logger);

        $attempt = 0;
        $pdo->transaction(function (\PDO $conn) use (&$attempt): void {
            $attempt++;
            if ($attempt === 1) {
                throw self::makeOccException();
            }
        });
    }

    // --- helper ---

    private static function makeOccException(): \PDOException
    {
        $e = new \PDOException('OC000 conflict detected');
        $ref = new \ReflectionProperty(\PDOException::class, 'code');
        $ref->setValue($e, '40001');
        return $e;
    }
}
