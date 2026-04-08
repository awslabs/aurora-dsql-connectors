<?php

// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

declare(strict_types=1);

namespace Aws\AuroraDsql\PdoPgsql\Tests\Integration;

class OccRetryConcurrentTest extends IntegrationTestBase
{
    public function testConcurrentIncrementsWithOccRetry(): void
    {
        $pdo = $this->createConnection();
        $tableName = $this->generateTableName('occ_concurrent');

        try {
            // Create table
            $this->createTestTable($pdo, $tableName);

            // Insert row with value=0
            $stmt = $pdo->prepare(
                sprintf('INSERT INTO %s (value) VALUES (0) RETURNING id', $tableName)
            );
            $stmt->execute();
            $rowId = $stmt->fetchColumn();

            // Create sync file for barrier synchronization
            $syncFile = sys_get_temp_dir() . '/occ_sync_' . getmypid() . '_' . bin2hex(random_bytes(4));
            touch($syncFile);

            // Spawn two concurrent workers
            $helperScript = __DIR__ . '/helpers/concurrent_increment.php';
            $phpBinary = PHP_BINARY;

            $descriptors = [
                0 => ['pipe', 'r'], // stdin
                1 => ['pipe', 'w'], // stdout
                2 => ['pipe', 'w'], // stderr
            ];

            $env = [
                'CLUSTER_ENDPOINT' => self::$clusterEndpoint,
                'REGION' => self::$region ?? '',
            ];

            $processes = [];
            for ($i = 0; $i < 2; $i++) {
                $cmd = sprintf(
                    '%s %s %s %s %d %s',
                    escapeshellarg($phpBinary),
                    escapeshellarg($helperScript),
                    escapeshellarg($tableName),
                    escapeshellarg($rowId),
                    $i,
                    escapeshellarg($syncFile)
                );

                $process = proc_open($cmd, $descriptors, $pipes, null, $env);
                $this->assertIsResource($process, "Failed to spawn worker $i");

                $processes[] = [
                    'process' => $process,
                    'pipes' => $pipes,
                    'index' => $i,
                ];
            }

            // Wait for all processes to complete
            $results = [];
            foreach ($processes as $p) {
                $stdout = stream_get_contents($p['pipes'][1]);
                $stderr = stream_get_contents($p['pipes'][2]);
                fclose($p['pipes'][0]);
                fclose($p['pipes'][1]);
                fclose($p['pipes'][2]);

                $exitCode = proc_close($p['process']);
                $results[] = [
                    'exit_code' => $exitCode,
                    'stdout' => trim($stdout),
                    'stderr' => trim($stderr),
                    'index' => $p['index'],
                ];
            }

            // Cleanup sync file
            unlink($syncFile);

            // Verify both workers succeeded (one via OCC retry)
            foreach ($results as $result) {
                $this->assertSame(
                    0,
                    $result['exit_code'],
                    sprintf(
                        "Worker %d failed with exit code %d\nSTDOUT: %s\nSTDERR: %s",
                        $result['index'],
                        $result['exit_code'],
                        $result['stdout'],
                        $result['stderr']
                    )
                );
                $this->assertSame('SUCCESS', $result['stdout'], "Worker {$result['index']} did not succeed");
            }

            // Verify final value is 2 (both increments applied)
            $stmt = $pdo->prepare(sprintf('SELECT value FROM %s WHERE id = ?', $tableName));
            $stmt->execute([$rowId]);
            $row = $stmt->fetch(\PDO::FETCH_ASSOC);

            $this->assertSame(
                '2',
                $row['value'],
                'Expected both concurrent increments to be applied (0 -> 1 -> 2)'
            );
        } finally {
            $this->dropTestTable($pdo, $tableName);
        }
    }

}
