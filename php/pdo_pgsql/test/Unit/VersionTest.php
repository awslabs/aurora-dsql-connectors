<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

namespace Aws\AuroraDsql\PdoPgsql\Tests\Unit;

use Aws\AuroraDsql\PdoPgsql\DsqlException;
use Aws\AuroraDsql\PdoPgsql\Version;
use PHPUnit\Framework\TestCase;

class VersionTest extends TestCase
{
    public function testGetVersionReturnsString(): void
    {
        $version = Version::getVersion();
        $this->assertIsString($version);
        $this->assertNotEmpty($version);
    }

    public function testBuildApplicationNameWithoutPrefix(): void
    {
        $name = Version::buildApplicationName();
        $this->assertStringStartsWith('aurora-dsql-php-pdo-pgsql/', $name);
    }

    public function testBuildApplicationNameWithPrefix(): void
    {
        $name = Version::buildApplicationName('laravel');
        $this->assertStringStartsWith('laravel:aurora-dsql-php-pdo-pgsql/', $name);
    }

    public function testBuildApplicationNameWithEmptyPrefix(): void
    {
        $name = Version::buildApplicationName('');
        $this->assertStringStartsWith('aurora-dsql-php-pdo-pgsql/', $name);
        $this->assertStringNotContainsString(':', $name);
    }

    public function testBuildApplicationNameWithWhitespacePrefix(): void
    {
        $name = Version::buildApplicationName('  ');
        $this->assertStringStartsWith('aurora-dsql-php-pdo-pgsql/', $name);
        $this->assertStringNotContainsString(':', $name);
    }

    public function testBuildApplicationNameRejectsSemicolonInPrefix(): void
    {
        $this->expectException(DsqlException::class);
        Version::buildApplicationName('foo;sslmode=disable');
    }

    public function testBuildApplicationNameRejectsEqualsInPrefix(): void
    {
        $this->expectException(DsqlException::class);
        Version::buildApplicationName('foo=bar');
    }
}
