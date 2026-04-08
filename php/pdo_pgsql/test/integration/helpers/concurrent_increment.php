<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

require_once __DIR__ . '/../../../vendor/autoload.php';

use Aws\AuroraDsql\PdoPgsql\AuroraDsql;
use Aws\AuroraDsql\PdoPgsql\DsqlConfig;

// Read arguments: tableName, rowId, workerIndex, syncFile
if ($argc < 5) {
    fwrite(STDERR, "Usage: {$argv[0]} <tableName> <rowId> <workerIndex> <syncFile>\n");
    exit(1);
}

[$script, $tableName, $rowId, $workerIndex, $syncFile] = $argv;

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
    $pdo->transaction(function (\PDO $conn) use ($tableName, $rowId, $workerIndex, $syncFile): void {
        // Read current value
        $stmt = $conn->prepare(sprintf('SELECT value FROM %s WHERE id = ?', $tableName));
        $stmt->execute([$rowId]);
        $row = $stmt->fetch(PDO::FETCH_ASSOC);
        $currentValue = (int) $row['value'];

        // Synchronization barrier: write worker readiness
        file_put_contents($syncFile, "$workerIndex\n", FILE_APPEND | LOCK_EX);

        // Wait for both workers to be ready
        $maxWait = 10; // seconds
        $waited = 0;
        while (substr_count(file_get_contents($syncFile), "\n") < 2) {
            usleep(100000); // 100ms
            $waited += 0.1;
            if ($waited >= $maxWait) {
                throw new RuntimeException('Timeout waiting for workers to synchronize');
            }
        }

        // Small additional sleep to ensure both transactions have read before either commits
        usleep(200000); // 200ms

        // Increment value
        $stmt = $conn->prepare(sprintf('UPDATE %s SET value = ? WHERE id = ?', $tableName));
        $stmt->execute([$currentValue + 1, $rowId]);
    });

    echo "SUCCESS\n";
    exit(0);
} catch (Throwable $e) {
    fwrite(STDERR, "ERROR: {$e->getMessage()}\n");
    exit(1);
}
