<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

namespace Aws\AuroraDsql\PdoPgsql;

use Aws\Credentials\CredentialProvider;
use Aws\DSQL\AuthTokenGenerator;

/** @internal */
class Token
{
    public static function generate(
        string $host,
        string $region,
        string $user,
        int $expiresInSecs,
        ?\Closure $credentialsProvider,
        ?string $profile,
    ): string {
        $credentials = self::resolveCredentials($credentialsProvider, $profile);
        $generator = new AuthTokenGenerator($credentials);

        return self::generateWithGenerator(
            $generator,
            host: $host,
            region: $region,
            user: $user,
            expiresInSecs: $expiresInSecs,
        );
    }

    public static function generateWithGenerator(
        AuthTokenGenerator $generator,
        string $host,
        string $region,
        string $user,
        int $expiresInSecs,
    ): string {
        if ($user === 'admin') {
            return $generator->generateDbConnectAdminAuthToken($host, $region, $expiresInSecs);
        }

        return $generator->generateDbConnectAuthToken($host, $region, $expiresInSecs);
    }

    private static function resolveCredentials(?\Closure $credentialsProvider, ?string $profile): callable
    {
        if ($credentialsProvider !== null) {
            return $credentialsProvider;
        }

        if ($profile !== null) {
            return CredentialProvider::defaultProvider(['profile' => $profile]);
        }

        return CredentialProvider::defaultProvider();
    }
}
