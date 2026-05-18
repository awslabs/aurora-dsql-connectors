/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

import { jest, describe, test, expect } from "@jest/globals";
import { fromNodeProviderChain } from "@aws-sdk/credential-providers";
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

    test("should handle config object with connectionString property", async () => {
      const client = new AuroraDSQLClient({
        connectionString: `postgresql://${clusterEndpoint}`,
      });
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

    test("should connect with custom credentials provider", async () => {
      let providerCalled = false;
      const trackingProvider = async () => {
        providerCalled = true;
        return fromNodeProviderChain()();
      };

      const client = new AuroraDSQLClient({
        host: clusterEndpoint,
        user: "admin",
        customCredentialsProvider: trackingProvider,
      });
      await verifySuccessfulConnection(client);
      expect(providerCalled).toBe(true);
    });

    test("should connect with custom credentials identity", async () => {
      const credentials = await fromNodeProviderChain()();
      const client = new AuroraDSQLClient({
        host: clusterEndpoint,
        user: "admin",
        customCredentialsProvider: credentials,
      });
      await verifySuccessfulConnection(client);
    });

    test("should fail with invalid custom credentials provider", async () => {
      const invalidProvider = async () => ({
        accessKeyId: "INVALID_ACCESS_KEY",
        secretAccessKey: "INVALID_SECRET_KEY",
      });

      const client = new AuroraDSQLClient({
        host: clusterEndpoint,
        user: "admin",
        customCredentialsProvider: invalidProvider,
      });

      await expect(client.connect()).rejects.toThrow();
    });

    test("should fail with invalid custom credentials identity", async () => {
      const client = new AuroraDSQLClient({
        host: clusterEndpoint,
        user: "admin",
        customCredentialsProvider: {
          accessKeyId: "INVALID_ACCESS_KEY",
          secretAccessKey: "INVALID_SECRET_KEY",
        },
      });

      await expect(client.connect()).rejects.toThrow();
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

    test("should connect with custom credentials provider", async () => {
      let providerCalled = false;
      const trackingProvider = async () => {
        providerCalled = true;
        return fromNodeProviderChain()();
      };

      const pool = new AuroraDSQLPool({
        host: clusterEndpoint,
        user: "admin",
        customCredentialsProvider: trackingProvider,
      });

      try {
        const result = await pool.query("SELECT 1 as test_value");
        expect(result.rows[0].test_value).toBe(1);
        expect(providerCalled).toBe(true);
      } finally {
        await pool.end();
      }
    });

    test("should connect with custom credentials identity", async () => {
      const credentials = await fromNodeProviderChain()();
      const pool = new AuroraDSQLPool({
        host: clusterEndpoint,
        user: "admin",
        customCredentialsProvider: credentials,
      });

      try {
        const result = await pool.query("SELECT 1 as test_value");
        expect(result.rows[0].test_value).toBe(1);
      } finally {
        await pool.end();
      }
    });

    test("should fail with invalid custom credentials provider", async () => {
      const invalidProvider = async () => ({
        accessKeyId: "INVALID_ACCESS_KEY",
        secretAccessKey: "INVALID_SECRET_KEY",
      });

      const pool = new AuroraDSQLPool({
        host: clusterEndpoint,
        user: "admin",
        customCredentialsProvider: invalidProvider,
      });

      await expect(pool.query("SELECT 1")).rejects.toThrow();
    });

    test("should fail with invalid custom credentials identity", async () => {
      const pool = new AuroraDSQLPool({
        host: clusterEndpoint,
        user: "admin",
        customCredentialsProvider: {
          accessKeyId: "INVALID_ACCESS_KEY",
          secretAccessKey: "INVALID_SECRET_KEY",
        },
      });

      await expect(pool.query("SELECT 1")).rejects.toThrow();
    });
  });

  describe("Application Name", () => {
    test("should set default application_name", async () => {
      const client = new AuroraDSQLClient({
        host: clusterEndpoint,
        user: "admin",
        region: region,
      });

      try {
        await client.connect();
        const result = await client.query("SELECT current_setting('application_name') as app_name");
        const appName = result.rows[0].app_name;
        expect(appName).toBeTruthy();
        expect(appName).toMatch(/^aurora-dsql-nodejs-pg\/\d+\.\d+\.\d+/);
        console.log(`Application name: ${appName}`);
      } finally {
        await client.end();
      }
    });

    test("should set application_name with ORM prefix", async () => {
      const client = new AuroraDSQLClient({
        host: clusterEndpoint,
        user: "admin",
        region: region,
        application_name: "prisma",
      });

      try {
        await client.connect();
        const result = await client.query("SELECT current_setting('application_name') as app_name");
        const appName = result.rows[0].app_name;
        expect(appName).toBeTruthy();
        expect(appName).toMatch(/^prisma:aurora-dsql-nodejs-pg\/\d+\.\d+\.\d+/);
        console.log(`Application name with ORM prefix: ${appName}`);
      } finally {
        await client.end();
      }
    });
  });

  describe("OCC Retry", () => {
    test("should retry OCC conflicts with Client", async () => {
      const logger = {
        warn: (msg: string) => console.log(`[client] ${msg}`),
        error: (msg: string) => console.error(`[client] ${msg}`),
      };

      const client1 = new AuroraDSQLClient({
        host: clusterEndpoint,
        user: "admin",
        retry: { maxRetries: 5 },
        logger,
      });

      const client2 = new AuroraDSQLClient({
        host: clusterEndpoint,
        user: "admin",
        retry: { maxRetries: 5 },
        logger,
      });

      try {
        await client1.connect();
        await client2.connect();
        await client1.query("CREATE TABLE IF NOT EXISTS occ_test_client (id INT PRIMARY KEY, value INT)");
        await client1.query("INSERT INTO occ_test_client (id, value) VALUES (1, 0) ON CONFLICT (id) DO UPDATE SET value = 0");

        const updatePromises = [
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
        ];

        await Promise.all(updatePromises);

        const finalResult = await client1.query("SELECT value FROM occ_test_client WHERE id = 1");
        expect(finalResult.rows[0].value).toBe(2);
      } finally {
        await client1.query("DROP TABLE IF EXISTS occ_test_client");
        await client1.end();
        await client2.end();
      }
    });

    test("should retry OCC conflicts with Pool", async () => {
      const logger = {
        warn: (msg: string) => console.log(`[pool] ${msg}`),
        error: (msg: string) => console.error(`[pool] ${msg}`),
      };

      const pool = new AuroraDSQLPool({
        host: clusterEndpoint,
        user: "admin",
        max: 5,
        retry: { maxRetries: 10 },
        logger,
      });

      try {
        await pool.query("CREATE TABLE IF NOT EXISTS occ_test_pool (id INT PRIMARY KEY, value INT)");
        await pool.query("INSERT INTO occ_test_pool (id, value) VALUES (1, 0) ON CONFLICT (id) DO UPDATE SET value = 0");

        const updatePromises = Array.from({ length: 10 }, () =>
          pool.transaction(async (client) => {
            const result = await client.query("SELECT value FROM occ_test_pool WHERE id = 1");
            const currentValue = result.rows[0].value;
            await client.query("UPDATE occ_test_pool SET value = $1 WHERE id = 1", [currentValue + 1]);
          })
        );

        await Promise.all(updatePromises);

        const finalResult = await pool.query("SELECT value FROM occ_test_pool WHERE id = 1");
        expect(finalResult.rows[0].value).toBe(10);
      } finally {
        await pool.query("DROP TABLE IF EXISTS occ_test_pool");
        await pool.end();
      }
    });

    test("should not retry non-OCC errors", async () => {
      const client = new AuroraDSQLClient({
        host: clusterEndpoint,
        user: "admin",
        retry: { maxRetries: 3 },
      });

      try {
        await client.connect();

        await expect(
          client.transaction(async (c) => {
            await c.query("SELECT * FROM nonexistent_table");
          })
        ).rejects.toThrow();
      } finally {
        await client.end();
      }
    });
  });
});
