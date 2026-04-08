<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

namespace Aws\AuroraDsql\PdoPgsql\Tests\Unit;

use Aws\AuroraDsql\PdoPgsql\AuroraDsql;
use Aws\AuroraDsql\PdoPgsql\DsqlConfig;
use Aws\AuroraDsql\PdoPgsql\DsqlException;
use PHPUnit\Framework\TestCase;

class ConnectTest extends TestCase
{
    public function testConnectFromDsnRejectsInvalidScheme(): void
    {
        $this->expectException(DsqlException::class);
        AuroraDsql::connectFromDsn('mysql://admin@localhost/test');
    }

    public function testConnectThrowsOnMissingRegion(): void
    {
        $origRegion = getenv('AWS_REGION');
        $origDefault = getenv('AWS_DEFAULT_REGION');
        putenv('AWS_REGION');
        putenv('AWS_DEFAULT_REGION');

        try {
            $config = new DsqlConfig(host: 'some-random-host.example.com');
            $this->expectException(DsqlException::class);
            $this->expectExceptionMessage('region');
            AuroraDsql::connect($config);
        } finally {
            if ($origRegion !== false) {
                putenv("AWS_REGION=$origRegion");
            }
            if ($origDefault !== false) {
                putenv("AWS_DEFAULT_REGION=$origDefault");
            }
        }
    }
}
