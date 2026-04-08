<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

namespace Aws\AuroraDsql\PdoPgsql\Tests\Unit;

use Aws\AuroraDsql\PdoPgsql\DsqlConfig;
use Aws\AuroraDsql\PdoPgsql\DsqlException;
use PHPUnit\Framework\TestCase;

class ConfigTest extends TestCase
{
    // --- DsqlConfig defaults ---

    public function testDefaultValues(): void
    {
        $config = new DsqlConfig(host: 'abc.dsql.us-east-1.on.aws');
        $this->assertSame('admin', $config->user);
        $this->assertSame('postgres', $config->database);
        $this->assertSame(5432, $config->port);
        $this->assertNull($config->region);
        $this->assertNull($config->profile);
        $this->assertNull($config->credentialsProvider);
        $this->assertSame(900, $config->tokenDurationSecs);
        $this->assertNull($config->ormPrefix);
    }

    // --- resolve() ---

    public function testResolveWithFullHostname(): void
    {
        $config = new DsqlConfig(host: 'abc.dsql.us-east-1.on.aws');
        $resolved = $config->resolve();

        $this->assertSame('abc.dsql.us-east-1.on.aws', $resolved->host);
        $this->assertSame('us-east-1', $resolved->region);
        $this->assertSame('admin', $resolved->user);
        $this->assertSame('postgres', $resolved->database);
        $this->assertSame(5432, $resolved->port);
    }

    public function testResolveWithClusterIdAndRegion(): void
    {
        $config = new DsqlConfig(
            host: 'abcdefghijklmnopqrstuvwxyz',
            region: 'eu-west-1',
        );
        $resolved = $config->resolve();

        $this->assertSame('abcdefghijklmnopqrstuvwxyz.dsql.eu-west-1.on.aws', $resolved->host);
        $this->assertSame('eu-west-1', $resolved->region);
    }

    public function testResolveWithClusterIdNoRegionThrows(): void
    {
        $origRegion = getenv('AWS_REGION');
        $origDefault = getenv('AWS_DEFAULT_REGION');
        putenv('AWS_REGION');
        putenv('AWS_DEFAULT_REGION');

        try {
            $config = new DsqlConfig(host: 'abcdefghijklmnopqrstuvwxyz');
            $this->expectException(DsqlException::class);
            $this->expectExceptionMessage('region');
            $config->resolve();
        } finally {
            if ($origRegion !== false) {
                putenv("AWS_REGION=$origRegion");
            }
            if ($origDefault !== false) {
                putenv("AWS_DEFAULT_REGION=$origDefault");
            }
        }
    }

    public function testResolveExplicitRegionOverridesHostParsing(): void
    {
        $config = new DsqlConfig(
            host: 'abc.dsql.us-east-1.on.aws',
            region: 'eu-west-1',
        );
        $resolved = $config->resolve();

        $this->assertSame('eu-west-1', $resolved->region);
    }

    public function testResolveApplicationNameDefault(): void
    {
        $config = new DsqlConfig(host: 'abc.dsql.us-east-1.on.aws');
        $resolved = $config->resolve();

        $this->assertStringStartsWith('aurora-dsql-php-pdo-pgsql/', $resolved->applicationName);
    }

    public function testResolveApplicationNameWithOrmPrefix(): void
    {
        $config = new DsqlConfig(
            host: 'abc.dsql.us-east-1.on.aws',
            ormPrefix: 'laravel',
        );
        $resolved = $config->resolve();

        $this->assertStringStartsWith('laravel:aurora-dsql-php-pdo-pgsql/', $resolved->applicationName);
    }

    // --- ResolvedConfig::toDsn() ---

    public function testToDsnContainsSslVerifyFull(): void
    {
        $config = new DsqlConfig(host: 'abc.dsql.us-east-1.on.aws');
        $dsn = $config->resolve()->toDsn();

        $this->assertStringContainsString('sslmode=verify-full', $dsn);
    }

    public function testToDsnFormat(): void
    {
        $config = new DsqlConfig(
            host: 'abc.dsql.us-east-1.on.aws',
            database: 'postgres',
            port: 5432,
        );
        $dsn = $config->resolve()->toDsn();

        $this->assertStringStartsWith('pgsql:', $dsn);
        $this->assertStringContainsString('host=abc.dsql.us-east-1.on.aws', $dsn);
        $this->assertStringContainsString('port=5432', $dsn);
        $this->assertStringContainsString('dbname=postgres', $dsn);
    }

    public function testToDsnIncludesApplicationName(): void
    {
        $config = new DsqlConfig(host: 'abc.dsql.us-east-1.on.aws');
        $dsn = $config->resolve()->toDsn();

        $this->assertStringContainsString('application_name=aurora-dsql-php-pdo-pgsql/', $dsn);
    }

    // --- ResolvedConfig DSN injection validation ---

    public function testResolveRejectsHostWithSemicolon(): void
    {
        $this->expectException(DsqlException::class);
        $this->expectExceptionMessage('host must not contain DSN special characters');
        $config = new DsqlConfig(
            host: 'evil;host=attacker.com',
            region: 'us-east-1',
        );
        $config->resolve();
    }

    public function testResolveRejectsHostWithEquals(): void
    {
        $this->expectException(DsqlException::class);
        $this->expectExceptionMessage('host must not contain DSN special characters');
        $config = new DsqlConfig(
            host: 'evil=host',
            region: 'us-east-1',
        );
        $config->resolve();
    }

    public function testResolveRejectsDatabaseWithSemicolon(): void
    {
        $this->expectException(DsqlException::class);
        $this->expectExceptionMessage('database must not contain DSN special characters');
        $config = new DsqlConfig(
            host: 'abc.dsql.us-east-1.on.aws',
            database: 'mydb;host=attacker.com',
        );
        $config->resolve();
    }

    public function testResolveRejectsDatabaseWithEquals(): void
    {
        $this->expectException(DsqlException::class);
        $this->expectExceptionMessage('database must not contain DSN special characters');
        $config = new DsqlConfig(
            host: 'abc.dsql.us-east-1.on.aws',
            database: 'mydb=bad',
        );
        $config->resolve();
    }

    // --- DsqlConfig::parse() ---

    public function testParsePostgresUri(): void
    {
        $config = DsqlConfig::parse(
            'postgres://myuser@abc.dsql.us-east-1.on.aws/mydb?region=eu-west-1&profile=dev&tokenDurationSecs=600'
        );

        $this->assertSame('abc.dsql.us-east-1.on.aws', $config->host);
        $this->assertSame('myuser', $config->user);
        $this->assertSame('mydb', $config->database);
        $this->assertSame('eu-west-1', $config->region);
        $this->assertSame('dev', $config->profile);
        $this->assertSame(600, $config->tokenDurationSecs);
    }

    public function testParsePostgresqlScheme(): void
    {
        $config = DsqlConfig::parse(
            'postgresql://admin@abc.dsql.us-east-1.on.aws/postgres'
        );

        $this->assertSame('abc.dsql.us-east-1.on.aws', $config->host);
        $this->assertSame('admin', $config->user);
    }

    public function testParseWithPortInUri(): void
    {
        $config = DsqlConfig::parse(
            'postgres://admin@abc.dsql.us-east-1.on.aws:9999/postgres'
        );

        $this->assertSame(9999, $config->port);
    }

    public function testParseRejectsInvalidScheme(): void
    {
        $this->expectException(DsqlException::class);
        DsqlConfig::parse('mysql://admin@localhost/test');
    }

    public function testParseWithOrmPrefix(): void
    {
        $config = DsqlConfig::parse(
            'postgres://admin@abc.dsql.us-east-1.on.aws/postgres?ormPrefix=laravel'
        );

        $this->assertSame('laravel', $config->ormPrefix);
    }

    public function testParseMinimalUri(): void
    {
        $config = DsqlConfig::parse(
            'postgres://abc.dsql.us-east-1.on.aws'
        );

        $this->assertSame('abc.dsql.us-east-1.on.aws', $config->host);
        $this->assertSame('admin', $config->user);
        $this->assertSame('postgres', $config->database);
    }

    public function testParseRejectsUnrecognizedQueryParams(): void
    {
        $this->expectException(DsqlException::class);
        $this->expectExceptionMessage('Unrecognized connection string parameter(s): sslmode');
        DsqlConfig::parse(
            'postgres://admin@abc.dsql.us-east-1.on.aws/postgres?sslmode=disable'
        );
    }

    public function testParseRejectsMultipleUnrecognizedQueryParams(): void
    {
        $this->expectException(DsqlException::class);
        $this->expectExceptionMessage('Unrecognized connection string parameter(s)');
        DsqlConfig::parse(
            'postgres://admin@abc.dsql.us-east-1.on.aws/postgres?foo=bar&baz=qux'
        );
    }

    // --- occMaxRetries ---

    public function testDefaultOccMaxRetriesIsNull(): void
    {
        $config = new DsqlConfig(host: 'abc.dsql.us-east-1.on.aws');
        $this->assertNull($config->occMaxRetries);
    }

    public function testOccMaxRetriesFlowsToResolvedConfig(): void
    {
        $config = new DsqlConfig(
            host: 'abc.dsql.us-east-1.on.aws',
            occMaxRetries: 5,
        );
        $resolved = $config->resolve();
        $this->assertSame(5, $resolved->occMaxRetries);
    }

    public function testNegativeOccMaxRetriesThrows(): void
    {
        $this->expectException(DsqlException::class);
        $this->expectExceptionMessage('occMaxRetries');
        $config = new DsqlConfig(
            host: 'abc.dsql.us-east-1.on.aws',
            occMaxRetries: -1,
        );
        $config->resolve();
    }

    public function testParseOccMaxRetriesFromDsn(): void
    {
        $config = DsqlConfig::parse(
            'postgres://admin@abc.dsql.us-east-1.on.aws/postgres?occMaxRetries=3'
        );
        $this->assertSame(3, $config->occMaxRetries);
    }

    public function testParseOccMaxRetriesAbsentIsNull(): void
    {
        $config = DsqlConfig::parse(
            'postgres://admin@abc.dsql.us-east-1.on.aws/postgres'
        );
        $this->assertNull($config->occMaxRetries);
    }
}
