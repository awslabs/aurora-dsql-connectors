/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import { AuroraDSQLClient } from "../src/aurora-dsql-client";
import { AuroraDSQLUtil } from "../src/aurora-dsql-util";
import { Client } from "pg";
import { AwsCredentialIdentity } from "@smithy/types";

jest.mock("pg");
jest.mock("../src/aurora-dsql-util");

const mockClient = Client as jest.MockedClass<typeof Client>;
const mockAuroraDSQLUtil = AuroraDSQLUtil as jest.Mocked<typeof AuroraDSQLUtil>;
const mockCredentials: AwsCredentialIdentity = {
  accessKeyId: "mockAccessKey",
  secretAccessKey: "mockSecretKey",
};

describe("AuroraDSQLClient", () => {
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
    mockAuroraDSQLUtil.getDSQLToken.mockResolvedValue("mock-token-123");
  });

  describe("constructor", () => {
    it("should throw error when config is undefined", () => {
      expect(() => new AuroraDSQLClient()).toThrow("Configuration is required");
    });

    it("should create client with string config", () => {
      const connectionString =
        "postgresql://admin@example.dsql.us-east-1.on.aws:5432/postgres";
      const client = new AuroraDSQLClient(connectionString);

      expect(mockAuroraDSQLUtil.parsePgConfig).toHaveBeenCalledWith(
        connectionString,
      );
      expect(client).toBeInstanceOf(AuroraDSQLClient);
      expect(mockClient).toHaveBeenCalledWith(
        expect.objectContaining({
          host: "example.dsql.us-east-1.on.aws",
          port: 5432,
          database: "postgres",
          user: "admin",
        }),
      );
    });

    it("should create client with config object", () => {
      const config = {
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
      };

      const client = new AuroraDSQLClient(config);

      expect(mockAuroraDSQLUtil.parsePgConfig).toHaveBeenCalledWith(config);
      expect(client).toBeInstanceOf(AuroraDSQLClient);
      expect(mockClient).toHaveBeenCalledWith(
        expect.objectContaining({
          host: "example.dsql.us-east-1.on.aws",
          user: "admin",
        }),
      );
    });

    it("should create client with clusterId and region", () => {
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

      const client = new AuroraDSQLClient(config);

      expect(mockAuroraDSQLUtil.parsePgConfig).toHaveBeenCalledWith(config);
      expect(client).toBeInstanceOf(AuroraDSQLClient);
      expect(mockClient).toHaveBeenCalledWith(
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

      expect(() => new AuroraDSQLClient({ user: "admin" } as any)).toThrow(
        "Host is required",
      );
    });

    it("should override defaults with user config", () => {
      new AuroraDSQLClient({
        host: "example.dsql.us-east-1.on.aws",
        user: "testuser",
        port: 3306,
        database: "mydb",
        profile: "custom-profile",
      });

      expect(mockClient).toHaveBeenCalledWith(
        expect.objectContaining({
          port: 3306,
          database: "mydb",
          profile: "custom-profile",
        }),
      );
    });
  });

  describe("connect", () => {
    let mockConnect: jest.Mock;
    let client: AuroraDSQLClient;

    beforeEach(() => {
      mockConnect = jest.fn().mockResolvedValue(undefined);
      mockClient.prototype.connect = mockConnect;

      client = new AuroraDSQLClient({
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
      });
    });

    it("should generate token and connect successfully", async () => {
      await client.connect();

      expect(mockAuroraDSQLUtil.getDSQLToken).toHaveBeenCalledWith(
        "example.dsql.us-east-1.on.aws",
        "admin",
        "default",
        "us-east-1",
        undefined,
        undefined,
      );
      expect(client.password).toBe("mock-token-123");
      expect(mockConnect).toHaveBeenCalled();
    });

    it("should handle connect with callback on success", (done) => {
      mockConnect.mockImplementation((cb) => {
        if (cb) cb(null);
        return Promise.resolve();
      });

      const callback = jest.fn((err) => {
        expect(err).toBeNull();
        expect(mockAuroraDSQLUtil.getDSQLToken).toHaveBeenCalled();
        expect(mockConnect).toHaveBeenCalledWith(callback);
        done();
      });

      client.connect(callback);
    });

    it("should handle token generation error with callback", (done) => {
      const tokenError = new Error("Token generation failed");
      mockAuroraDSQLUtil.getDSQLToken.mockRejectedValue(tokenError);

      const callback = jest.fn((err) => {
        expect(err).toBe(tokenError);
        expect(mockConnect).not.toHaveBeenCalled();
        done();
      });

      client.connect(callback);
    });

    it("should throw token generation error without callback", async () => {
      const tokenError = new Error("Token generation failed");
      mockAuroraDSQLUtil.getDSQLToken.mockRejectedValue(tokenError);

      await expect(client.connect()).rejects.toThrow("Token generation failed");
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
        tokenDurationSecs: 15,
        customCredentialsProvider: mockCredentials,
      });

      const customClient = new AuroraDSQLClient({
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
      });

      await customClient.connect();

      expect(mockAuroraDSQLUtil.getDSQLToken).toHaveBeenCalledWith(
        "example.dsql.us-east-1.on.aws",
        "admin",
        "custom-profile",
        "us-east-1",
        15,
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

      const euClient = new AuroraDSQLClient({
        host: "cluster.dsql.eu-west-1.on.aws",
        user: "admin",
      });

      await euClient.connect();

      expect(mockAuroraDSQLUtil.getDSQLToken).toHaveBeenCalledWith(
        "cluster.dsql.eu-west-1.on.aws",
        "admin",
        "default",
        "eu-west-1",
        undefined,
        undefined,
      );
    });

    it("should handle non-admin users", async () => {
      mockAuroraDSQLUtil.parsePgConfig.mockReturnValueOnce({
        host: "example.dsql.us-east-1.on.aws",
        user: "testuser",
        port: 5432,
        database: "postgres",
        region: "us-east-1",
        profile: "default",
        ssl: { rejectUnauthorized: true },
      });

      const userClient = new AuroraDSQLClient({
        host: "example.dsql.us-east-1.on.aws",
        user: "testuser",
      });

      await userClient.connect();

      expect(mockAuroraDSQLUtil.getDSQLToken).toHaveBeenCalledWith(
        "example.dsql.us-east-1.on.aws",
        "testuser",
        "default",
        "us-east-1",
        undefined,
        undefined,
      );
    });
  });

  describe("OCC retry integration", () => {
    let client: AuroraDSQLClient;
    let mockQuery: jest.Mock;

    beforeEach(() => {
      mockQuery = jest.fn();
      mockClient.prototype.query = mockQuery;

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

      client = new AuroraDSQLClient({
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

      const result = await client.query("SELECT * FROM accounts");

      expect(mockQuery).toHaveBeenCalledTimes(2);
      expect(result.rows).toEqual([{ id: 1 }]);
    });

    it("should skip retry when skipRetry is true", async () => {
      const occError = new Error("conflict") as any;
      occError.code = "OC000";

      mockQuery.mockRejectedValueOnce(occError);

      await expect(client.query({ text: "SELECT 1", skipRetry: true })).rejects.toThrow("conflict");
      expect(mockQuery).toHaveBeenCalledTimes(1);
    });

    it("should not retry on non-OCC errors", async () => {
      const syntaxError = new Error("syntax error") as any;
      syntaxError.code = "42601";

      mockQuery.mockRejectedValueOnce(syntaxError);

      await expect(client.query("INVALID SQL")).rejects.toThrow("syntax error");
      expect(mockQuery).toHaveBeenCalledTimes(1);
    });

    it("should retry transaction on OCC conflict", async () => {
      const occError = new Error("conflict") as any;
      occError.code = "OC000";

      let attempts = 0;
      mockQuery.mockImplementation(async (sql: string) => {
        if (sql === 'BEGIN') return { rows: [] };
        if (sql === 'COMMIT') {
          attempts++;
          if (attempts === 1) throw occError;
          return { rows: [] };
        }
        if (sql === 'ROLLBACK') return { rows: [] };
        return { rows: [{ id: 1 }] };
      });

      const result = await client.transactionWithRetry(async (c) => {
        await c.query("INSERT INTO accounts VALUES(1)");
        return "success";
      });

      expect(result).toBe("success");
      expect(attempts).toBe(2);
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
        occ: { enabled: true, maxAttempts: 0 }
      });

      expect(() => new AuroraDSQLClient({
        host: "example.dsql.us-east-1.on.aws",
        user: "admin",
        occ: { enabled: true, maxAttempts: 0 }
      })).toThrow('occ.maxAttempts must be between 1 and 100');
    });
  });
});
