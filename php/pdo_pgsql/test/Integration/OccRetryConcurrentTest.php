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

            // Spawn two concurrent workers that will race to increment the same row
            $helperScript = __DIR__ . '/helpers/concurrent_increment.php';
            $phpBinary = PHP_BINARY;

            $descriptors = [
                0 => ['pipe', 'r'], // stdin
                1 => ['pipe', 'w'], // stdout
                2 => ['pipe', 'w'], // stderr
            ];

            // Pass through AWS credentials to child processes
            $env = [
                'CLUSTER_ENDPOINT' => self::$clusterEndpoint,
                'REGION' => self::$region ?? '',
            ];

            // Pass AWS credentials if set (needed for IAM authentication)
            foreach (['AWS_ACCESS_KEY_ID', 'AWS_SECRET_ACCESS_KEY', 'AWS_SESSION_TOKEN', 'AWS_REGION'] as $key) {
                if ($value = getenv($key)) {
                    $env[$key] = $value;
                }
            }

            $processes = [];
            for ($i = 0; $i < 2; $i++) {
                $cmd = sprintf(
                    '%s %s %s %s %d',
                    escapeshellarg($phpBinary),
                    escapeshellarg($helperScript),
                    escapeshellarg($tableName),
                    escapeshellarg($rowId),
                    $i
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
            // This proves OCC retry worked - one worker hit conflict and retried successfully
            $stmt = $pdo->prepare(sprintf('SELECT value FROM %s WHERE id = ?', $tableName));
            $stmt->execute([$rowId]);
            $row = $stmt->fetch(\PDO::FETCH_ASSOC);

            $this->assertSame(
                '2',
                $row['value'],
                'Expected both concurrent increments to be applied (0 -> 1 -> 2). ' .
                'If this fails, OCC retry did not work correctly.'
            );
        } finally {
            $this->dropTestTable($pdo, $tableName);
        }
    }

}
