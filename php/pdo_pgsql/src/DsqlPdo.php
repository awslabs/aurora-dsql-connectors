<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

namespace Aws\AuroraDsql\PdoPgsql;

use Psr\Log\LoggerInterface;

/**
 * PDO subclass with automatic Optimistic Concurrency Control (OCC) retry.
 *
 * Extends PDO with built-in retry logic for Aurora DSQL OCC conflicts
 * (SQLSTATE 40001, OC000, OC001). Supports retry for both single statements
 * via exec() and multi-statement transactions via transaction().
 *
 * Use AuroraDsql::connect() to create instances rather than constructing directly.
 *
 * @see AuroraDsql::connect()
 */
class DsqlPdo extends \PDO
{
    private const INITIAL_WAIT_MS = 100;
    private const MAX_WAIT_MS = 5000;
    private const MULTIPLIER = 2.0;

    /**
     * @internal Use AuroraDsql::connect() to create instances.
     */
    public function __construct(
        string $dsn,
        ?string $username = null,
        ?string $password = null,
        ?array $options = null,
        private readonly int $occMaxRetries = 0,
        private readonly ?LoggerInterface $logger = null,
    ) {
        parent::__construct($dsn, $username, $password, $options);
    }

    /**
     * Execute a SQL statement with automatic OCC retry.
     *
     * When OCC retries are enabled and this call is NOT inside a manual
     * transaction, single statements are retried on OCC conflict with
     * exponential backoff — no explicit transaction wrapping is applied.
     * This is suitable for DDL and single DML statements.
     *
     * Inside a transaction (via transaction() or manual beginTransaction()),
     * this delegates to PDO::exec() without retry — the enclosing
     * transaction() handles retries at the transaction level.
     *
     * WARNING: Retried statements must be idempotent. If a statement has
     * side effects that are not safe to repeat, use transaction() instead.
     */
    public function exec(string $statement): int|false
    {
        if ($this->occMaxRetries === 0 || $this->inTransaction()) {
            return parent::exec($statement);
        }

        return self::retry(
            fn() => parent::exec($statement),
            $this->occMaxRetries,
            $this->logger,
        );
    }

    /**
     * Execute a callback inside a transaction with automatic OCC retry.
     *
     * Handles beginTransaction()/commit()/rollBack() internally.
     * The callback receives this PDO instance and should NOT call
     * beginTransaction() or commit() — that is managed here.
     *
     * On OCC conflict (SQLSTATE 40001, OC000, OC001), the transaction is
     * rolled back and the callback is re-executed with exponential backoff.
     *
     * @template T
     * @param callable(\PDO): T $callback
     * @param ?int $maxRetries Override default from DsqlConfig (null = use default)
     * @return T
     * @throws DsqlException After max retries exhausted
     */
    public function transaction(callable $callback, ?int $maxRetries = null): mixed
    {
        $retries = $maxRetries ?? $this->occMaxRetries;
        $logger = $this->logger;

        return self::retry(
            function () use ($callback, $logger): mixed {
                $this->beginTransaction();
                try {
                    $result = $callback($this);
                    $this->commit();
                    return $result;
                } catch (\Throwable $e) {
                    if ($this->inTransaction()) {
                        try {
                            $this->rollBack();
                        } catch (\PDOException $rollbackEx) {
                            $msg = sprintf(
                                '[AuroraDsql] rollBack() failed during OCC retry: %s',
                                $rollbackEx->getMessage(),
                            );
                            if ($logger !== null) {
                                $logger->error($msg);
                            } else {
                                error_log($msg);
                            }
                        }
                    }
                    throw $e;
                }
            },
            $retries,
            $logger,
        );
    }

    /**
     * Retries a callable on OCC conflict with exponential backoff + jitter.
     *
     * @template T
     * @param callable(): T $fn
     * @return T
     * @throws DsqlException After max retries exhausted
     */
    private static function retry(
        callable $fn,
        int $maxRetries,
        ?LoggerInterface $logger,
    ): mixed {
        $waitMs = self::INITIAL_WAIT_MS;

        for ($attempt = 0; $attempt <= $maxRetries; $attempt++) {
            try {
                return $fn();
            } catch (\Throwable $e) {
                // Throw immediately if not an OCC error, or if retries are disabled (maxRetries === 0)
                if (!OCCRetry::isOccError($e) || $maxRetries === 0) {
                    throw $e;
                }

                if ($attempt === $maxRetries) {
                    throw new DsqlException(
                        "OCC max retries ({$maxRetries}) exceeded",
                        $e,
                    );
                }

                $jitter = random_int(0, $waitMs);
                $sleepMs = $waitMs + $jitter;

                $logger?->warning(sprintf(
                    '[AuroraDsql] OCC conflict detected, retrying (attempt %d/%d, wait %.2fs)',
                    $attempt + 1,
                    $maxRetries,
                    $sleepMs / 1000.0,
                ));

                usleep($sleepMs * 1000);

                $waitMs = (int) min($waitMs * self::MULTIPLIER, self::MAX_WAIT_MS);
            }
        }

        // Unreachable, but satisfies static analysis
        throw new DsqlException("OCC max retries ({$maxRetries}) exceeded");
    }
}
