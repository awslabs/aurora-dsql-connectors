/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

import assert from "node:assert";
import { AuroraDSQLPool } from "@aws/aurora-dsql-node-postgres-connector";

const NUM_CONCURRENT_QUERIES = 8;

function createPool(clusterEndpoint, user) {
  return new AuroraDSQLPool({
    host: clusterEndpoint,
    user: user,
    max: 10,
    idleTimeoutMillis: 30000,
    connectionTimeoutMillis: 10000,
    occ: {
      enabled: true,        // Enable automatic retry for all queries
      maxAttempts: 3,       // Optional (default: 3)
      baseDelayMs: 1,       // Optional (default: 1)
      maxDelayMs: 100,      // Optional (default: 100)
      jitterFactor: 0.25    // Optional (default: 0.25)
    }
  });
}

async function worker(pool, workerId) {
  const result = await pool.query("SELECT $1::int as worker_id", [workerId]);
  console.log(`Worker ${workerId} result: ${result.rows[0].worker_id}`);
  assert.strictEqual(result.rows[0].worker_id, workerId);
}

async function example() {
  const clusterEndpoint = process.env.CLUSTER_ENDPOINT;
  assert(clusterEndpoint, "CLUSTER_ENDPOINT environment variable is not set");
  const user = process.env.CLUSTER_USER;
  assert(user, "CLUSTER_USER environment variable is not set");

  const pool = createPool(clusterEndpoint, user);

  try {
    // OCC retry is enabled - all queries automatically retry on conflicts

    // Run concurrent queries using the connection pool
    const workers = [];
    for (let i = 1; i <= NUM_CONCURRENT_QUERIES; i++) {
      workers.push(worker(pool, i));
    }

    // Wait for all workers to complete
    await Promise.all(workers);

    // Opt-out per query using QueryConfig
    await pool.query({ text: "SELECT 1", skipRetry: true });

    console.log("Connection pool with concurrent connections exercised successfully");
  } catch (error) {
    console.error(error);
    throw error;
  } finally {
    await pool.end();
  }
}

export { example };
