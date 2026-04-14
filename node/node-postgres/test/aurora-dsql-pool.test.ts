/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import { AuroraDSQLPool } from "../src/aurora-dsql-pool";
import { AuroraDSQLUtil } from "../src/aurora-dsql-util";
import { Pool, PoolClient } from "pg";
import { AwsCredentialIdentity } from "@smithy/types";

jest.mock("pg");
jest.mock("../src/aurora-dsql-util");

const mockPool = Pool as jest.MockedClass<typeof Pool>;
const mockAuroraDSQLUtil = AuroraDSQLUtil as jest.Mocked<typeof AuroraDSQLUtil>;
const mockCredentials: AwsCredentialIdentity = {
  accessKeyId: "mockAccessKey",
  secretAccessKey: "mockSecretKey",
};

describe("AuroraDSQLPool", () => {
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
    mockAuroraDSQLUtil.getDSQLToken.mockResolvedValue("mock-pool-token-456");
  });

  describe("constructor", () => {
    it("should throw error when config is undefined", () => {
      expect(() => new AuroraDSQLPool()).toThrow("Configuration is required");
    });

    it("should create pool with valid config", () => {
      const config = {
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
        max: 10,
        min: 2,
      };

      mockAuroraDSQLUtil.parsePgConfig.mockReturnValueOnce({
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
        port: 5432,
        database: "postgres",
        region: "us-east-1",
        profile: "default",
        ssl: { rejectUnauthorized: true },
        max: 10,
        min: 2,
      });

      const pool = new AuroraDSQLPool(config);

      expect(mockAuroraDSQLUtil.parsePgConfig).toHaveBeenCalledWith(config);
      expect(mockPool).toHaveBeenCalledWith(
        expect.objectContaining({
          host: "example.dsql.us-east-1.on.aws",
          user: "admin",
          max: 10,
          min: 2,
          port: 5432,
          database: "postgres",
          region: "us-east-1",
          profile: "default",
          ssl: { rejectUnauthorized: true },
        }),
      );
      expect(pool).toBeInstanceOf(AuroraDSQLPool);
    });

    it("should create pool with clusterId and region", () => {
      const config = {
        host: "cluster123",
        user: "admin",
        region: "us-west-2",
      };

      mockAuroraDSQLUtil.parsePgConfig.mockReturnValueOnce({
        host: "cluster123.dsql.us-west-2.on.aws",
        user: "admin",
        port: 5432,
        database: "postgres",
        region: "us-west-2",
        profile: "default",
        ssl: { rejectUnauthorized: true },
      });

      const pool = new AuroraDSQLPool(config);

      expect(mockAuroraDSQLUtil.parsePgConfig).toHaveBeenCalledWith(config);
      expect(pool).toBeInstanceOf(AuroraDSQLPool);
      expect(mockPool).toHaveBeenCalledWith(
        expect.objectContaining({
          host: "cluster123.dsql.us-west-2.on.aws",
          user: "admin",
          port: 5432,
          database: "postgres",
          region: "us-west-2",
          profile: "default",
        }),
      );
    });

    it("should throw error from validatePgConfig when host is missing", () => {
      mockAuroraDSQLUtil.parsePgConfig.mockImplementation(() => {
        throw new Error("Host is required");
      });

      expect(() => new AuroraDSQLPool({ user: "admin" })).toThrow(
        "Host is required",
      );
    });

    it("should override defaults with user config", () => {
      mockAuroraDSQLUtil.parsePgConfig.mockReturnValueOnce({
        host: "example.dsql.us-east-1.on.aws",
        user: "testuser",
        port: 3306,
        database: "mydb",
        region: "us-east-1",
        profile: "custom-profile",
        ssl: { rejectUnauthorized: true },
        max: 20,
        idleTimeoutMillis: 30000,
      });

      new AuroraDSQLPool({
        host: "example.dsql.us-east-1.on.aws",
        user: "testuser",
        port: 3306,
        database: "mydb",
        profile: "custom-profile",
        max: 20,
        idleTimeoutMillis: 30000,
      });

      expect(mockPool).toHaveBeenCalledWith(
        expect.objectContaining({
          port: 3306,
          database: "mydb",
          profile: "custom-profile",
          max: 20,
          idleTimeoutMillis: 30000,
        }),
      );
    });
  });

  describe("connect", () => {
    let mockConnect: jest.Mock;
    let pool: AuroraDSQLPool;
    let mockClient: PoolClient;

    beforeEach(() => {
      mockClient = {} as PoolClient;
      mockConnect = jest.fn().mockResolvedValue(mockClient);
      mockPool.prototype.connect = mockConnect;

      pool = new AuroraDSQLPool({
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
      });

      // Mock pool options
      (pool as any).options = {
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
        port: 5432,
      };
    });

    it("should generate token and connect successfully", async () => {
      const result = await pool.connect();

      expect(mockAuroraDSQLUtil.getDSQLToken).toHaveBeenCalledWith(
        "example.dsql.us-east-1.on.aws",
        "admin",
        "default",
        "us-east-1",
        undefined,
        undefined,
      );
      expect((pool as any).options.password).toBe("mock-pool-token-456");
      expect(mockConnect).toHaveBeenCalled();
      expect(result).toBe(mockClient);
    });

    it("should handle connect with callback", (done) => {
      mockConnect.mockImplementation((cb) => {
        if (cb) cb(undefined, mockClient, jest.fn());
        return Promise.resolve(mockClient);
      });

      const callback = jest.fn((err, client, release) => {
        expect(err).toBeUndefined();
        expect(client).toBe(mockClient);
        expect(typeof release).toBe("function");
        expect(mockAuroraDSQLUtil.getDSQLToken).toHaveBeenCalled();
        expect(mockConnect).toHaveBeenCalledWith(callback);
        done();
      });

      pool.connect(callback);
    });

    it("should handle token generation error with callback", (done) => {
      const tokenError = new Error("Pool token generation failed");
      mockAuroraDSQLUtil.getDSQLToken.mockRejectedValue(tokenError);

      const callback = jest.fn((err, client, release) => {
        expect(err).toBe(tokenError);
        expect(client).toBeUndefined();
        expect(typeof release).toBe("function");
        expect(mockConnect).not.toHaveBeenCalled();
        done();
      });

      pool.connect(callback);
    });

    it("should throw token generation error without callback", async () => {
      const tokenError = new Error("Pool token generation failed");
      mockAuroraDSQLUtil.getDSQLToken.mockRejectedValue(tokenError);

      await expect(pool.connect()).rejects.toThrow(
        "Pool token generation failed",
      );
      expect(mockConnect).not.toHaveBeenCalled();
    });

    it("should pass config to the token generator", async () => {
      mockAuroraDSQLUtil.parsePgConfig.mockReturnValueOnce({
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
        port: 5432,
        database: "postgres",
        region: "us-east-1",
        profile: "custom-profile",
        ssl: { rejectUnauthorized: true },
        tokenDurationSecs: 5,
        customCredentialsProvider: mockCredentials,
      });

      const customPool = new AuroraDSQLPool({
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
      });

      (customPool as any).options = {
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
      };

      await customPool.connect();

      expect(mockAuroraDSQLUtil.getDSQLToken).toHaveBeenCalledWith(
        "example.dsql.us-east-1.on.aws",
        "admin",
        "custom-profile",
        "us-east-1",
        5,
        mockCredentials,
      );
    });

    it("should handle different regions", async () => {
      mockAuroraDSQLUtil.parsePgConfig.mockReturnValueOnce({
        host: "cluster.dsql.eu-west-1.on.aws",
        user: "admin",
        port: 5432,
        database: "postgres",
        region: "eu-west-1",
        profile: "default",
        ssl: { rejectUnauthorized: true },
      });

      const euPool = new AuroraDSQLPool({
        host: "cluster.dsql.eu-west-1.on.aws",
        user: "admin",
      });

      (euPool as any).options = {
        host: "cluster.dsql.eu-west-1.on.aws",
        user: "admin",
      };

      await euPool.connect();

      expect(mockAuroraDSQLUtil.getDSQLToken).toHaveBeenCalledWith(
        "cluster.dsql.eu-west-1.on.aws",
        "admin",
        "default",
        "eu-west-1",
        undefined,
        undefined,
      );
    });

    it("should pass tokenDurationSecs to getDSQLToken", async () => {
      mockAuroraDSQLUtil.parsePgConfig.mockReturnValueOnce({
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
        port: 5432,
        database: "postgres",
        region: "us-east-1",
        profile: "default",
        ssl: { rejectUnauthorized: true },
        tokenDurationSecs: 1800,
      });

      const poolWithDuration = new AuroraDSQLPool({
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
        tokenDurationSecs: 1800,
      });

      (poolWithDuration as any).options = {
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
      };

      await poolWithDuration.connect();

      expect(mockAuroraDSQLUtil.getDSQLToken).toHaveBeenCalledWith(
        "example.dsql.us-east-1.on.aws",
        "admin",
        "default",
        "us-east-1",
        1800,
        undefined,
      );
    });

    it("should throw error when options is undefined", async () => {
      Object.defineProperty(pool, "options", {
        value: undefined,
        writable: true,
        configurable: true,
      });

      await expect(pool.connect()).rejects.toThrow(
        "options is undefined in this pool",
      );
      expect(mockConnect).not.toHaveBeenCalled();
    });

    it("should handle pool-specific options", async () => {
      mockAuroraDSQLUtil.parsePgConfig.mockReturnValueOnce({
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
        port: 5432,
        database: "postgres",
        region: "us-east-1",
        profile: "default",
        ssl: { rejectUnauthorized: true },
        max: 15,
        min: 3,
        idleTimeoutMillis: 20000,
      });

      const poolWithOptions = new AuroraDSQLPool({
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
        max: 15,
        min: 3,
        idleTimeoutMillis: 20000,
      });

      (poolWithOptions as any).options = {
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
      };

      await poolWithOptions.connect();

      expect(mockConnect).toHaveBeenCalled();
      expect(mockPool).toHaveBeenCalledWith(
        expect.objectContaining({
          port: 5432,
          database: "postgres",
          profile: "default",
          max: 15,
          min: 3,
          idleTimeoutMillis: 20000,
        }),
      );
    });
  });

  describe("OCC retry integration", () => {
    let pool: AuroraDSQLPool;
    let mockQuery: jest.Mock;

    beforeEach(() => {
      mockQuery = jest.fn();
      mockPool.prototype.query = mockQuery;

      mockAuroraDSQLUtil.parsePgConfig.mockReturnValueOnce({
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
        port: 5432,
        database: "postgres",
        region: "us-east-1",
        profile: "default",
        ssl: { rejectUnauthorized: true },
        occ: { enabled: true, maxAttempts: 3, baseDelayMs: 1, maxDelayMs: 10, jitterFactor: 0 }
      });

      pool = new AuroraDSQLPool({
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
        occ: { enabled: true, maxAttempts: 3, baseDelayMs: 1, maxDelayMs: 10, jitterFactor: 0 }
      });
    });

    it("should retry query on OCC conflict and succeed", async () => {
      const occError = new Error("conflict") as any;
      occError.code = "OC000";

      mockQuery
        .mockRejectedValueOnce(occError)
        .mockResolvedValueOnce({ rows: [{ id: 1 }] });

      const result = await pool.query("INSERT INTO accounts VALUES($1)", [1]);

      expect(mockQuery).toHaveBeenCalledTimes(2);
      expect(result.rows).toEqual([{ id: 1 }]);
    });

    it("should skip retry when skipRetry is true", async () => {
      const occError = new Error("conflict") as any;
      occError.code = "OC000";

      mockQuery.mockRejectedValueOnce(occError);

      await expect(pool.query({ text: "SELECT 1", skipRetry: true })).rejects.toThrow("conflict");
      expect(mockQuery).toHaveBeenCalledTimes(1);
    });

    it("should not retry on non-OCC errors", async () => {
      const uniqueViolation = new Error("unique violation") as any;
      uniqueViolation.code = "23505";

      mockQuery.mockRejectedValueOnce(uniqueViolation);

      await expect(pool.query("INSERT INTO accounts VALUES(1)")).rejects.toThrow("unique violation");
      expect(mockQuery).toHaveBeenCalledTimes(1);
    });

    it("should retry transaction on OCC conflict", async () => {
      const occError = new Error("conflict") as any;
      occError.code = "OC000";

      const mockConnect = jest.fn();
      const mockClient = {
        query: jest.fn(),
        release: jest.fn()
      };

      mockPool.prototype.connect = mockConnect;
      mockConnect.mockResolvedValue(mockClient);

      // Mock pool options
      (pool as any).options = {
        host: "example.dsql.us-east-1.on.aws",
        user: "admin"
      };

      mockAuroraDSQLUtil.getDSQLToken.mockResolvedValue("mock-token");

      let attempts = 0;
      mockClient.query.mockImplementation(async (sql: string) => {
        if (sql === 'BEGIN') return { rows: [] };
        if (sql === 'COMMIT') {
          attempts++;
          if (attempts === 1) throw occError;
          return { rows: [] };
        }
        if (sql === 'ROLLBACK') return { rows: [] };
        return { rows: [{ id: 1 }] };
      });

      const result = await pool.transactionWithRetry(async (c) => {
        await c.query("INSERT INTO accounts VALUES(1)");
        return "success";
      });

      expect(result).toBe("success");
      expect(attempts).toBe(2);
      expect(mockClient.release).toHaveBeenCalled();
    });

    it("should validate occ config on initialization", () => {
      mockAuroraDSQLUtil.parsePgConfig.mockReturnValueOnce({
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
        port: 5432,
        database: "postgres",
        region: "us-east-1",
        profile: "default",
        ssl: { rejectUnauthorized: true },
        occ: { enabled: true, jitterFactor: 2 }
      });

      expect(() => new AuroraDSQLPool({
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
        occ: { enabled: true, jitterFactor: 2 }
      })).toThrow('occ.jitterFactor must be between 0 and 1');
    });
  });
});
