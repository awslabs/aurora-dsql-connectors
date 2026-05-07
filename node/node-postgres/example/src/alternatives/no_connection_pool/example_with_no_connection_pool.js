/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

import assert from "node:assert";
import { AuroraDSQLClient } from "@aws/aurora-dsql-node-postgres-connector";

const ADMIN = "admin";
const NON_ADMIN_SCHEMA = "myschema";

function createClient(clusterEndpoint, user) {
  return new AuroraDSQLClient({
    host: clusterEndpoint,
    user: user,
    transaction: {
      retry: { maxAttempts: 10, baseDelayMs: 10, jitter: true },
    },
  });
}

async function example() {
  const clusterEndpoint = process.env.CLUSTER_ENDPOINT;
  assert(clusterEndpoint, "CLUSTER_ENDPOINT environment variable is not set");
  const user = process.env.CLUSTER_USER;
  assert(user, "CLUSTER_USER environment variable is not set");

  const client1 = createClient(clusterEndpoint, user);
  const client2 = createClient(clusterEndpoint, user);
  await client1.connect();
  await client2.connect();

  try {
    if (user !== ADMIN) {
      await client1.query("SET search_path=" + NON_ADMIN_SCHEMA);
      await client2.query("SET search_path=" + NON_ADMIN_SCHEMA);
    }

    await client1.query("CREATE TABLE IF NOT EXISTS occ_test_client (id INT PRIMARY KEY, value INT)");
    await client1.query("INSERT INTO occ_test_client (id, value) VALUES (1, 0) ON CONFLICT (id) DO UPDATE SET value = 0");

    // Run concurrent transactional writes across two clients.
    // OCC conflicts are automatically retried by client.transaction().
    await Promise.all([
      client1.transaction(async (c) => {
        const result = await c.query("SELECT value FROM occ_test_client WHERE id = 1");
        const currentValue = result.rows[0].value;
        await c.query("UPDATE occ_test_client SET value = $1 WHERE id = 1", [currentValue + 1]);
      }),
      client2.transaction(async (c) => {
        const result = await c.query("SELECT value FROM occ_test_client WHERE id = 1");
        const currentValue = result.rows[0].value;
        await c.query("UPDATE occ_test_client SET value = $1 WHERE id = 1", [currentValue + 1]);
      }),
    ]);

    const { rows } = await client1.query("SELECT value FROM occ_test_client WHERE id = 1");
    assert.strictEqual(rows[0].value, 2);
    console.log(`Final counter value: ${rows[0].value} (expected 2)`);
    console.log("Client with OCC retry exercised successfully");
  } catch (error) {
    console.error(error);
    throw error;
  } finally {
    await client1.query("DROP TABLE IF EXISTS occ_test_client");
    await client1.end();
    await client2.end();
  }
}

export { example };
