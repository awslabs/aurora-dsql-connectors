<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

namespace Aws\AuroraDsql\PdoPgsql;

use Composer\InstalledVersions;

/** @internal */
class Version
{
    public const APPLICATION_NAME_BASE = 'aurora-dsql-php-pdo-pgsql';

    public static function getVersion(): string
    {
        if (class_exists(InstalledVersions::class)) {
            $version = InstalledVersions::getVersion('awslabs/aurora-dsql-pdo-pgsql');
            if ($version !== null) {
                return $version;
            }
        }

        return '0.0.0';
    }

    public static function buildApplicationName(?string $ormPrefix = null): string
    {
        $base = self::APPLICATION_NAME_BASE . '/' . self::getVersion();

        if ($ormPrefix !== null && trim($ormPrefix) !== '') {
            Util::validateNoDsnSpecialChars($ormPrefix, 'ormPrefix');
            return trim($ormPrefix) . ':' . $base;
        }

        return $base;
    }
}
