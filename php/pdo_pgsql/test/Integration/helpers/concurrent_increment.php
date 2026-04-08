<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

require_once __DIR__ . '/../../../vendor/autoload.php';

use Aws\AuroraDsql\PdoPgsql\AuroraDsql;
use Aws\AuroraDsql\PdoPgsql\DsqlConfig;

// Read arguments: tableName, rowId, workerIndex
if ($argc < 4) {
    fwrite(STDERR, "Usage: {$argv[0]} <tableName> <rowId> <workerIndex>\n");
    exit(1);
}

[$script, $tableName, $rowId, $workerIndex] = $argv;

$clusterEndpoint = getenv('CLUSTER_ENDPOINT');
if (!$clusterEndpoint) {
    fwrite(STDERR, "CLUSTER_ENDPOINT environment variable required\n");
    exit(1);
}

$region = getenv('REGION') ?: null;

try {
    $config = new DsqlConfig(
        host: $clusterEndpoint,
        region: $region,
        user: 'admin',
        occMaxRetries: 5,
    );
    $pdo = AuroraDsql::connect($config);

    // Execute concurrent increment with OCC retry
    $pdo->transaction(function (\PDO $conn) use ($tableName, $rowId): void {
        // Read current value
        $stmt = $conn->prepare(sprintf('SELECT value FROM %s WHERE id = ?', $tableName));
        $stmt->execute([$rowId]);
        $row = $stmt->fetch(PDO::FETCH_ASSOC);
        $currentValue = (int) $row['value'];

        // Sleep to increase likelihood of concurrent transaction overlap
        // This creates the race condition that triggers OCC conflicts
        usleep(200000); // 200ms

        // Increment value - one worker will get OCC error and retry
        $stmt = $conn->prepare(sprintf('UPDATE %s SET value = ? WHERE id = ?', $tableName));
        $stmt->execute([$currentValue + 1, $rowId]);
    });

    echo "SUCCESS\n";
    exit(0);
} catch (Throwable $e) {
    fwrite(STDERR, "ERROR: {$e->getMessage()}\n");
    exit(1);
}
