/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

import assert from "node:assert";
import { AuroraDSQLPool } from "@aws/aurora-dsql-node-postgres-connector";

const NUM_CONCURRENT_WORKERS = 8;

function createPool(clusterEndpoint, user) {
  return new AuroraDSQLPool({
    host: clusterEndpoint,
    user: user,
    max: 10,
    idleTimeoutMillis: 30000,
    connectionTimeoutMillis: 10000,
    transaction: {
      retry: { maxAttempts: 10, baseDelayMs: 10, jitter: true },
    },
  });
}

async function worker(pool, workerId) {
  await pool.transaction(async (client) => {
    const result = await client.query("SELECT value FROM occ_test WHERE id = 1");
    const currentValue = result.rows[0].value;
    await client.query("UPDATE occ_test SET value = $1 WHERE id = 1", [currentValue + 1]);
  });
  console.log(`Worker ${workerId} completed`);
}

async function example() {
  const clusterEndpoint = process.env.CLUSTER_ENDPOINT;
  assert(clusterEndpoint, "CLUSTER_ENDPOINT environment variable is not set");
  const user = process.env.CLUSTER_USER;
  assert(user, "CLUSTER_USER environment variable is not set");

  const pool = createPool(clusterEndpoint, user);

  try {
    await pool.query("CREATE TABLE IF NOT EXISTS occ_test (id INT PRIMARY KEY, value INT)");
    await pool.query("INSERT INTO occ_test (id, value) VALUES (1, 0) ON CONFLICT (id) DO UPDATE SET value = 0");

    // Run concurrent transactional writes.
    // OCC conflicts are automatically retried by pool.transaction().
    const workers = [];
    for (let i = 1; i <= NUM_CONCURRENT_WORKERS; i++) {
      workers.push(worker(pool, i));
    }
    await Promise.all(workers);

    const { rows } = await pool.query("SELECT value FROM occ_test WHERE id = 1");
    assert.strictEqual(rows[0].value, NUM_CONCURRENT_WORKERS);
    console.log(`Final counter value: ${rows[0].value} (expected ${NUM_CONCURRENT_WORKERS})`);
    console.log("Connection pool with OCC retry exercised successfully");
  } catch (error) {
    console.error(error);
    throw error;
  } finally {
    await pool.query("DROP TABLE IF EXISTS occ_test");
    await pool.end();
  }
}

export { example };
