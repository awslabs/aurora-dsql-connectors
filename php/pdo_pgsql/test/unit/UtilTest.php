<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

namespace Aws\AuroraDsql\PdoPgsql\Tests\Unit;

use Aws\AuroraDsql\PdoPgsql\Util;
use PHPUnit\Framework\TestCase;

class UtilTest extends TestCase
{
    public function testParseRegionFromStandardHost(): void
    {
        $this->assertSame(
            'us-east-1',
            Util::parseRegion('abc123.dsql.us-east-1.on.aws')
        );
    }

    public function testParseRegionFromSuffixHost(): void
    {
        $this->assertSame(
            'us-east-1',
            Util::parseRegion('abc123.dsql-gamma.us-east-1.on.aws')
        );
    }

    public function testParseRegionReturnsNullForNonDsqlHost(): void
    {
        $this->assertNull(Util::parseRegion('mydb.example.com'));
    }

    public function testParseRegionReturnsNullForClusterId(): void
    {
        $this->assertNull(Util::parseRegion('abcdefghijklmnopqrstuvwxyz'));
    }

    public function testIsClusterIdValid(): void
    {
        $this->assertTrue(Util::isClusterId('abcdefghijklmnopqrstuvwxyz'));
    }

    public function testIsClusterIdValid26AlphanumericChars(): void
    {
        $this->assertTrue(Util::isClusterId('abcdefghij1234567890abcdef'));
    }

    public function testIsClusterIdRejectsDots(): void
    {
        $this->assertFalse(Util::isClusterId('abc.dsql.us-east-1.on.aws'));
    }

    public function testIsClusterIdRejectsEmpty(): void
    {
        $this->assertFalse(Util::isClusterId(''));
    }

    public function testIsClusterIdRejectsUppercase(): void
    {
        $this->assertFalse(Util::isClusterId('ABCDEFGHIJKLMNOPQRSTUVWXYZ'));
    }

    public function testIsClusterIdRejectsWrongLength(): void
    {
        $this->assertFalse(Util::isClusterId('abc'));
    }

    public function testBuildHostname(): void
    {
        $this->assertSame(
            'mycluster.dsql.us-east-1.on.aws',
            Util::buildHostname('mycluster', 'us-east-1')
        );
    }

    public function testRegionFromEnvReadsAwsRegion(): void
    {
        $original = getenv('AWS_REGION');
        putenv('AWS_REGION=eu-west-1');
        putenv('AWS_DEFAULT_REGION=');

        try {
            $this->assertSame('eu-west-1', Util::regionFromEnv());
        } finally {
            if ($original !== false) {
                putenv("AWS_REGION=$original");
            } else {
                putenv('AWS_REGION');
            }
        }
    }

    public function testRegionFromEnvFallsBackToDefaultRegion(): void
    {
        $origRegion = getenv('AWS_REGION');
        $origDefault = getenv('AWS_DEFAULT_REGION');
        putenv('AWS_REGION');
        putenv('AWS_DEFAULT_REGION=ap-southeast-1');

        try {
            $this->assertSame('ap-southeast-1', Util::regionFromEnv());
        } finally {
            if ($origRegion !== false) {
                putenv("AWS_REGION=$origRegion");
            }
            if ($origDefault !== false) {
                putenv("AWS_DEFAULT_REGION=$origDefault");
            } else {
                putenv('AWS_DEFAULT_REGION');
            }
        }
    }

    public function testRegionFromEnvReturnsNullWhenUnset(): void
    {
        $origRegion = getenv('AWS_REGION');
        $origDefault = getenv('AWS_DEFAULT_REGION');
        putenv('AWS_REGION');
        putenv('AWS_DEFAULT_REGION');

        try {
            $this->assertNull(Util::regionFromEnv());
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
