<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

namespace Aws\AuroraDsql\PdoPgsql\Tests\Unit;

use Aws\AuroraDsql\PdoPgsql\Token;
use Aws\DSQL\AuthTokenGenerator;
use PHPUnit\Framework\TestCase;

class TokenTest extends TestCase
{
    public function testGenerateCallsAdminMethodForAdminUser(): void
    {
        $mockGenerator = $this->createMock(AuthTokenGenerator::class);
        $mockGenerator->expects($this->once())
            ->method('generateDbConnectAdminAuthToken')
            ->with('abc.dsql.us-east-1.on.aws', 'us-east-1', 900)
            ->willReturn('admin-token-123');

        $token = Token::generateWithGenerator(
            $mockGenerator,
            host: 'abc.dsql.us-east-1.on.aws',
            region: 'us-east-1',
            user: 'admin',
            expiresInSecs: 900,
        );

        $this->assertSame('admin-token-123', $token);
    }

    public function testGenerateCallsRegularMethodForNonAdminUser(): void
    {
        $mockGenerator = $this->createMock(AuthTokenGenerator::class);
        $mockGenerator->expects($this->once())
            ->method('generateDbConnectAuthToken')
            ->with('abc.dsql.us-east-1.on.aws', 'us-east-1', 900)
            ->willReturn('regular-token-456');

        $token = Token::generateWithGenerator(
            $mockGenerator,
            host: 'abc.dsql.us-east-1.on.aws',
            region: 'us-east-1',
            user: 'appuser',
            expiresInSecs: 900,
        );

        $this->assertSame('regular-token-456', $token);
    }
}
