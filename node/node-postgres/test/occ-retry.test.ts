/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import { AuroraDSQLPool } from "../src/aurora-dsql-pool";
import { AuroraDSQLClient } from "../src/aurora-dsql-client";
import { AuroraDSQLUtil } from "../src/aurora-dsql-util";
import { Pool, Client } from "pg";

jest.mock("pg");
jest.mock("../src/aurora-dsql-util");

const mockPool = Pool as jest.MockedClass<typeof Pool>;
const mockClient = Client as jest.MockedClass<typeof Client>;
const mockAuroraDSQLUtil = AuroraDSQLUtil as jest.Mocked<typeof AuroraDSQLUtil>;

describe("OCC Retry", () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockAuroraDSQLUtil.parsePgConfig.mockImplementation((config) => ({
      host: "example.dsql.us-east-1.on.aws",
      user: "admin",
      port: 5432,
      database: "postgres",
      region: "us-east-1",
      profile: "default",
      ssl: { rejectUnauthorized: true },
      ...(typeof config === "string" ? {} : config),
    }));
    mockAuroraDSQLUtil.getDSQLToken.mockResolvedValue("mock-token");
  });

  describe("Pool.transaction", () => {
    let pool: AuroraDSQLPool;
    let mockPoolClient: any;

    beforeEach(() => {
      mockPoolClient = {
        query: jest.fn().mockResolvedValue({ rows: [] }),
        release: jest.fn(),
      };
      mockPool.prototype.connect = jest.fn().mockResolvedValue(mockPoolClient);

      pool = new AuroraDSQLPool({
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
        transaction: { retry: { maxAttempts: 3 } },
      });

      (pool as any).options = {
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
      };
    });

    it("should execute transaction successfully without retry", async () => {
      const callback = jest.fn().mockResolvedValue("result");

      const result = await pool.transaction(callback);

      expect(result).toBe("result");
      expect(callback).toHaveBeenCalledTimes(1);
      expect(mockPoolClient.query).toHaveBeenCalledWith("BEGIN");
      expect(mockPoolClient.query).toHaveBeenCalledWith("COMMIT");
      expect(mockPoolClient.release).toHaveBeenCalled();
    });

    it("should retry on OC000 data conflict", async () => {
      const occError = Object.assign(new Error("OCC"), { code: "OC000" });
      const callback = jest.fn()
        .mockRejectedValueOnce(occError)
        .mockResolvedValue("success");

      const result = await pool.transaction(callback);

      expect(result).toBe("success");
      expect(callback).toHaveBeenCalledTimes(2);
    });

    it("should retry on OC001 schema conflict", async () => {
      const occError = Object.assign(new Error("OCC"), { code: "OC001" });
      const callback = jest.fn()
        .mockRejectedValueOnce(occError)
        .mockResolvedValue("success");

      const result = await pool.transaction(callback);

      expect(result).toBe("success");
      expect(callback).toHaveBeenCalledTimes(2);
    });

    it("should retry on 40001 serialization failure", async () => {
      const occError = Object.assign(
        new Error("serialization failure"),
        { code: "40001" },
      );
      const callback = jest.fn()
        .mockRejectedValueOnce(occError)
        .mockResolvedValue("success");

      const result = await pool.transaction(callback);

      expect(result).toBe("success");
      expect(callback).toHaveBeenCalledTimes(2);
    });

    it("should not retry non-OCC errors", async () => {
      const genericError = new Error("syntax error");
      const callback = jest.fn().mockRejectedValue(genericError);

      await expect(pool.transaction(callback)).rejects.toThrow("syntax error");
      expect(callback).toHaveBeenCalledTimes(1);
    });

    it("should throw after max attempts exhausted", async () => {
      const occError = Object.assign(new Error("OCC"), { code: "OC000" });
      const callback = jest.fn().mockRejectedValue(occError);

      await expect(pool.transaction(callback)).rejects.toThrow("OCC");
      expect(callback).toHaveBeenCalledTimes(3);
    });

    it("should not retry when disabled via options", async () => {
      const occError = Object.assign(new Error("OCC"), { code: "OC000" });
      const callback = jest.fn().mockRejectedValue(occError);

      await expect(pool.transaction(callback, { retry: false })).rejects.toThrow("OCC");
      expect(callback).toHaveBeenCalledTimes(1);
    });

    it("should override constructor config with per-call options", async () => {
      const occError = Object.assign(new Error("OCC"), { code: "OC000" });
      const callback = jest.fn().mockRejectedValue(occError);

      await expect(pool.transaction(callback, { retry: { maxAttempts: 5 } })).rejects.toThrow("OCC");
      expect(callback).toHaveBeenCalledTimes(5);
    });

    it("should release pool client after each attempt", async () => {
      const occError = Object.assign(new Error("OCC"), { code: "OC000" });
      const callback = jest.fn()
        .mockRejectedValueOnce(occError)
        .mockResolvedValue("success");

      await pool.transaction(callback);

      expect(mockPoolClient.release).toHaveBeenCalledTimes(2);
    });

    it("should rollback on error before retrying", async () => {
      const occError = Object.assign(new Error("OCC"), { code: "OC000" });
      const callback = jest.fn()
        .mockRejectedValueOnce(occError)
        .mockResolvedValue("success");

      await pool.transaction(callback);

      expect(mockPoolClient.query).toHaveBeenCalledWith("ROLLBACK");
    });

    it("should still retry and release client when ROLLBACK fails", async () => {
      const occError = Object.assign(new Error("OCC"), { code: "OC000" });
      mockPoolClient.query = jest.fn().mockImplementation((sql: string) => {
        if (sql === "ROLLBACK") return Promise.reject(new Error("connection lost"));
        return Promise.resolve({ rows: [] });
      });
      const callback = jest.fn()
        .mockRejectedValueOnce(occError)
        .mockResolvedValue("success");

      const result = await pool.transaction(callback);

      expect(result).toBe("success");
      expect(callback).toHaveBeenCalledTimes(2);
      expect(mockPoolClient.release).toHaveBeenCalledTimes(2);
    });
  });

  describe("Client.transaction", () => {
    let client: AuroraDSQLClient;
    let mockQuery: jest.Mock;

    beforeEach(() => {
      mockQuery = jest.fn().mockResolvedValue({ rows: [] });
      mockClient.prototype.query = mockQuery;
      mockClient.prototype.connect = jest.fn().mockResolvedValue(undefined);

      client = new AuroraDSQLClient({
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
        transaction: { retry: { maxAttempts: 3 } },
      });
    });

    it("should execute transaction successfully without retry", async () => {
      const callback = jest.fn().mockResolvedValue("result");

      const result = await client.transaction(callback);

      expect(result).toBe("result");
      expect(callback).toHaveBeenCalledTimes(1);
      expect(mockQuery).toHaveBeenCalledWith("BEGIN");
      expect(mockQuery).toHaveBeenCalledWith("COMMIT");
    });

    it("should retry on OC000 data conflict", async () => {
      const occError = Object.assign(new Error("OCC"), { code: "OC000" });
      const callback = jest.fn()
        .mockRejectedValueOnce(occError)
        .mockResolvedValue("success");

      const result = await client.transaction(callback);

      expect(result).toBe("success");
      expect(callback).toHaveBeenCalledTimes(2);
    });

    it("should retry on OC001 schema conflict", async () => {
      const occError = Object.assign(new Error("OCC"), { code: "OC001" });
      const callback = jest.fn()
        .mockRejectedValueOnce(occError)
        .mockResolvedValue("success");

      const result = await client.transaction(callback);

      expect(result).toBe("success");
      expect(callback).toHaveBeenCalledTimes(2);
    });

    it("should not retry non-OCC errors", async () => {
      const genericError = new Error("syntax error");
      const callback = jest.fn().mockRejectedValue(genericError);

      await expect(client.transaction(callback)).rejects.toThrow("syntax error");
      expect(callback).toHaveBeenCalledTimes(1);
    });

    it("should throw after max attempts exhausted", async () => {
      const occError = Object.assign(new Error("OCC"), { code: "OC000" });
      const callback = jest.fn().mockRejectedValue(occError);

      await expect(client.transaction(callback)).rejects.toThrow("OCC");
      expect(callback).toHaveBeenCalledTimes(3);
    });

    it("should not retry when disabled via options", async () => {
      const occError = Object.assign(new Error("OCC"), { code: "OC000" });
      const callback = jest.fn().mockRejectedValue(occError);

      await expect(client.transaction(callback, { retry: false })).rejects.toThrow("OCC");
      expect(callback).toHaveBeenCalledTimes(1);
    });

    it("should override constructor config with per-call options", async () => {
      const occError = Object.assign(new Error("OCC"), { code: "OC000" });
      const callback = jest.fn().mockRejectedValue(occError);

      await expect(client.transaction(callback, { retry: { maxAttempts: 5 } })).rejects.toThrow("OCC");
      expect(callback).toHaveBeenCalledTimes(5);
    });

    it("should rollback on error before retrying", async () => {
      const occError = Object.assign(new Error("OCC"), { code: "OC000" });
      const callback = jest.fn()
        .mockRejectedValueOnce(occError)
        .mockResolvedValue("success");

      await client.transaction(callback);

      expect(mockQuery).toHaveBeenCalledWith("ROLLBACK");
    });
  });

  describe("retry config validation", () => {
    let pool: AuroraDSQLPool;

    beforeEach(() => {
      const mockPoolClient = {
        query: jest.fn().mockResolvedValue({ rows: [] }),
        release: jest.fn(),
      };
      mockPool.prototype.connect = jest.fn().mockResolvedValue(mockPoolClient);

      pool = new AuroraDSQLPool({
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
      });

      (pool as any).options = {
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
      };
    });

    it("should reject maxAttempts <= 0", async () => {
      const callback = jest.fn();
      await expect(
        pool.transaction(callback, { retry: { maxAttempts: 0 } }),
      ).rejects.toThrow("maxAttempts must be a positive integer");
    });

    it("should reject non-integer maxAttempts", async () => {
      const callback = jest.fn();
      await expect(
        pool.transaction(callback, { retry: { maxAttempts: 2.5 } }),
      ).rejects.toThrow("maxAttempts must be a positive integer");
    });

    it("should reject baseDelayMs <= 0", async () => {
      const callback = jest.fn();
      await expect(
        pool.transaction(callback, { retry: { baseDelayMs: 0 } }),
      ).rejects.toThrow("baseDelayMs must be a positive number");
    });

    it("should reject maxDelayMs <= 0", async () => {
      const callback = jest.fn();
      await expect(
        pool.transaction(callback, { retry: { maxDelayMs: 0 } }),
      ).rejects.toThrow("maxDelayMs must be a positive number");
    });

    it("should reject maxDelayMs < baseDelayMs", async () => {
      const callback = jest.fn();
      await expect(
        pool.transaction(callback, { retry: { baseDelayMs: 50, maxDelayMs: 10 } }),
      ).rejects.toThrow("maxDelayMs must be >= baseDelayMs");
    });
  });
});
