<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

namespace Aws\AuroraDsql\PdoPgsql;

/**
 * Exception thrown for Aurora DSQL connector configuration and connection errors.
 *
 * Thrown when:
 * - Configuration is invalid (missing region, invalid hostname format)
 * - Connection string parsing fails
 * - OCC retry limit is exceeded
 * - Token generation fails
 */
class DsqlException extends \RuntimeException
{
    public function __construct(string $message, ?\Throwable $previous = null)
    {
        parent::__construct($message, 0, $previous);
    }
}
