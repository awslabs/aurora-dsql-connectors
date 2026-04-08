<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

namespace Aws\AuroraDsql\PdoPgsql\Tests\Unit;

use Aws\AuroraDsql\PdoPgsql\OCCRetry;
use PHPUnit\Framework\TestCase;

class OCCRetryTest extends TestCase
{
    // --- isOccError ---

    public function testIsOccErrorWithSqlstate40001(): void
    {
        $e = new \PDOException('Serialization failure');
        $ref = new \ReflectionProperty(\PDOException::class, 'code');
        $ref->setValue($e, '40001');

        $this->assertTrue(OCCRetry::isOccError($e));
    }

    public function testIsOccErrorWithOC000InMessage(): void
    {
        $e = new \PDOException('ERROR: OC000 conflict detected');
        $this->assertTrue(OCCRetry::isOccError($e));
    }

    public function testIsOccErrorWithOC001InMessage(): void
    {
        $e = new \PDOException('ERROR: OC001 schema conflict');
        $this->assertTrue(OCCRetry::isOccError($e));
    }

    public function testIsOccErrorReturnsFalseForOtherErrors(): void
    {
        $e = new \PDOException('Connection refused');
        $this->assertFalse(OCCRetry::isOccError($e));
    }

    public function testIsOccErrorReturnsFalseForNonPdoException(): void
    {
        $e = new \RuntimeException('some error');
        $this->assertFalse(OCCRetry::isOccError($e));
    }

    public function testIsOccErrorReturnsTrueForNonPdoExceptionWithOccMessage(): void
    {
        // ORM layers (Doctrine, Eloquent) wrap PDOException in their own types
        // but preserve OCC error codes in the message — these should still match.
        $e = new \RuntimeException('ERROR: OC000 conflict detected');
        $this->assertTrue(OCCRetry::isOccError($e));
    }
}
