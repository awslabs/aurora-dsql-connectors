<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

namespace Aws\AuroraDsql\PdoPgsql;

/**
 * Utility for detecting Optimistic Concurrency Control (OCC) errors.
 *
 * Identifies Aurora DSQL OCC conflicts by checking for SQLSTATE codes:
 * - 40001 (serialization failure)
 * - OC000 (DSQL-specific OCC conflict)
 * - OC001 (DSQL-specific OCC conflict)
 *
 * Supports both direct PDOException instances and ORM-wrapped exceptions
 * that preserve error codes in the message.
 */
class OCCRetry
{
    /**
     * Check if an exception represents an OCC conflict.
     *
     * @param \Throwable $error The exception to check
     * @return bool True if the error is an OCC conflict
     */
    public static function isOccError(\Throwable $error): bool
    {
        if ($error instanceof \PDOException) {
            $code = (string) $error->getCode();
            if ($code === '40001' || $code === 'OC000' || $code === 'OC001') {
                return true;
            }
            // Don't return false — fall through to message matching below
        }

        // Fallback to message matching for all Throwable types — ORM layers
        // (Doctrine, Eloquent) wrap PDOException in their own exception types
        // but preserve the OCC error codes in the message.
        $message = $error->getMessage();
        return (str_contains($message, 'SQLSTATE') && str_contains($message, '40001'))
            || str_contains($message, 'OC000')
            || str_contains($message, 'OC001');
    }
}
