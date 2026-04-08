<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

namespace Aws\AuroraDsql\PdoPgsql;

/** @internal */
class Util
{
    private const REGION_PATTERN = '/\.dsql[^.]*\.([^.]+)\.on\.aws$/';
    private const CLUSTER_ID_PATTERN = '/^[a-z0-9]{26}$/';
    private const DSN_SPECIAL_CHARS = '/[;=]/';

    public static function validateNoDsnSpecialChars(string $value, string $fieldName): void
    {
        $result = preg_match(self::DSN_SPECIAL_CHARS, $value);
        if ($result === false) {
            throw new DsqlException("Regex validation failed for {$fieldName} (PCRE error: " . preg_last_error() . ')');
        }
        if ($result === 1) {
            throw new DsqlException("{$fieldName} must not contain DSN special characters (;, =)");
        }
    }

    public static function parseRegion(string $host): ?string
    {
        if (preg_match(self::REGION_PATTERN, $host, $matches)) {
            return $matches[1];
        }

        return null;
    }

    public static function isClusterId(string $host): bool
    {
        return (bool) preg_match(self::CLUSTER_ID_PATTERN, $host);
    }

    public static function buildHostname(string $clusterId, string $region): string
    {
        return "{$clusterId}.dsql.{$region}.on.aws";
    }

    public static function regionFromEnv(): ?string
    {
        $region = getenv('AWS_REGION');
        if ($region !== false && $region !== '') {
            return $region;
        }

        $region = getenv('AWS_DEFAULT_REGION');
        if ($region !== false && $region !== '') {
            return $region;
        }

        return null;
    }
}
