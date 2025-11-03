/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

import { jest, describe, test, expect } from "@jest/globals";
import { AuroraDSQLClient } from "../../src/aurora-dsql-client";
import { AuroraDSQLPool } from "../../src/aurora-dsql-pool";

jest.setTimeout(30000);

async function verifySuccessfulConnection(client: AuroraDSQLClient) {
  try {
    await client.connect();
    const result = await client.query("SELECT 1 as test_value");
    expect(result.rows[0].test_value).toBe(1);
  } finally {
    await client.end();
  }
}

describe("DSQL Integration Tests", () => {
  const clusterEndpoint = process.env.CLUSTER_ENDPOINT;
  const region = process.env.REGION;

  describe("AuroraDSQLClient", () => {
    test("should connect to DSQL cluster", async () => {
      const client = new AuroraDSQLClient({
        host: clusterEndpoint,
        database: "postgres",
        user: "admin",
        region: region,
        port: 5432,
      });
      await verifySuccessfulConnection(client);
    });

    test("should connect without providing region", async () => {
      const client = new AuroraDSQLClient({
        host: clusterEndpoint,
        database: "postgres",
        user: "admin",
        port: 5432,
      });
      await verifySuccessfulConnection(client);
    });

    test("should connect with minimum parameters", async () => {
      const client = new AuroraDSQLClient({
        host: clusterEndpoint,
        user: "admin",
      });
      await verifySuccessfulConnection(client);
    });

    test("should handle connection string format", async () => {
      const connectionString = `postgresql://admin@${clusterEndpoint}:5432/postgres`;
      const client = new AuroraDSQLClient(connectionString);
      await verifySuccessfulConnection(client);
    });

    test("should handle parameterized queries", async () => {
      const client = new AuroraDSQLClient({
        host: clusterEndpoint,
        user: "admin",
        region: region,
      });

      try {
        await client.connect();
        const result = await client.query("SELECT $1 as param_value", [42]);
        expect(result.rows[0].param_value).toBe("42");
      } finally {
        await client.end();
      }
    });

    test("should connect with non-admin user", async () => {
      const client = new AuroraDSQLClient({
        host: clusterEndpoint,
        user: "testuser",
        region: region,
      });

      try {
        await client.connect();
        const result = await client.query("SELECT current_user as username");
        expect(result.rows[0].username).toBe("testuser");
      } finally {
        await client.end();
      }
    });
  });

  describe("AuroraDSQLPool", () => {
    test("should handle concurrent queries with pool", async () => {
      const pool = new AuroraDSQLPool({
        host: clusterEndpoint,
        user: "admin",
        region: region,
        max: 3,
      });

      try {
        const promises = [
          pool.query("SELECT 1 as value"),
          pool.query("SELECT 2 as value"),
          pool.query("SELECT 3 as value"),
        ];

        const results = await Promise.all(promises);
        expect(results[0].rows[0].value).toBe(1);
        expect(results[1].rows[0].value).toBe(2);
        expect(results[2].rows[0].value).toBe(3);
      } finally {
        await pool.end();
      }
    });

    test("should connect pool with minimum parameters", async () => {
      const pool = new AuroraDSQLPool({
        host: clusterEndpoint,
        user: "admin",
      });

      try {
        const result = await pool.query("SELECT 1 as test_value");
        expect(result.rows[0].test_value).toBe(1);
      } finally {
        await pool.end();
      }
    });
  });
});
