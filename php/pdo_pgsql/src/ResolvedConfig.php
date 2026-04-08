<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

namespace Aws\AuroraDsql\PdoPgsql;

use Psr\Log\LoggerInterface;

/** @internal */
class ResolvedConfig
{
    public function __construct(
        public readonly string $host,
        public readonly string $region,
        public readonly string $user,
        public readonly string $database,
        public readonly int $port,
        public readonly ?string $profile,
        public readonly ?\Closure $credentialsProvider,
        public readonly int $tokenDurationSecs,
        public readonly string $applicationName,
        public readonly ?int $occMaxRetries,
        public readonly ?LoggerInterface $logger,
    ) {
        Util::validateNoDsnSpecialChars($host, 'host');
        Util::validateNoDsnSpecialChars($database, 'database');
        Util::validateNoDsnSpecialChars($applicationName, 'applicationName');
    }

    public function toDsn(): string
    {
        $params = [
            'host' => $this->host,
            'port' => (string) $this->port,
            'dbname' => $this->database,
            'sslmode' => 'verify-full',
            'application_name' => $this->applicationName,
        ];

        $parts = [];
        foreach ($params as $key => $value) {
            $parts[] = "{$key}={$value}";
        }

        return 'pgsql:' . implode(';', $parts);
    }
}
