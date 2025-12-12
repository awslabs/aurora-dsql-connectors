/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import { AuroraDSQLUtil } from "../src/aurora-dsql-util";
import { DsqlSigner } from "@aws-sdk/dsql-signer";
import { fromNodeProviderChain } from "@aws-sdk/credential-providers";

jest.mock("@aws-sdk/dsql-signer");
jest.mock("@aws-sdk/credential-providers");

const mockDsqlSigner = DsqlSigner as jest.MockedClass<typeof DsqlSigner>;
const mockFromNodeProviderChain = fromNodeProviderChain as jest.MockedFunction<
  typeof fromNodeProviderChain
>;

describe("AuroraDSQLUtil", () => {
  beforeEach(() => {
    jest.clearAllMocks();
    delete process.env.AWS_REGION;
    delete process.env.AWS_DEFAULT_REGION;
  });

  describe("parseRegion", () => {
    it("should parse region from valid DSQL hostname", () => {
      const host = "cluster123.dsql.us-east-1.on.aws";
      expect(AuroraDSQLUtil.parseRegion(host)).toBe("us-east-1");
    });

    it("should parse region from different regions", () => {
      expect(AuroraDSQLUtil.parseRegion("cluster.dsql.eu-west-1.on.aws")).toBe(
        "eu-west-1"
      );
      expect(AuroraDSQLUtil.parseRegion("cluster.dsql.ap-south-1.on.aws")).toBe(
        "ap-south-1"
      );
    });

    it("should throw error when hostname is empty", () => {
      expect(() => AuroraDSQLUtil.parseRegion("")).toThrow(
        "Hostname is required to parse region"
      );
    });

    it("should throw error when hostname format is invalid", () => {
      expect(() => AuroraDSQLUtil.parseRegion("invalid-hostname")).toThrow(
        "Unable to parse region from hostname"
      );
    });
  });

  describe("getDSQLToken", () => {
    const mockCredential = { accessKeyId: "key", secretAccessKey: "secret" };
    const mockToken = "mock-token-123";

    beforeEach(() => {
      mockFromNodeProviderChain.mockReturnValue(mockCredential as any);
      mockDsqlSigner.prototype.getDbConnectAdminAuthToken = jest
        .fn()
        .mockResolvedValue(mockToken);
      mockDsqlSigner.prototype.getDbConnectAuthToken = jest
        .fn()
        .mockResolvedValue(mockToken);
    });

    it("should generate admin token for admin user", async () => {
      const token = await AuroraDSQLUtil.getDSQLToken(
        "cluster.dsql.us-east-1.on.aws",
        "admin",
        "default",
        "us-east-1"
      );

      expect(token).toBe(mockToken);
      expect(
        mockDsqlSigner.prototype.getDbConnectAdminAuthToken
      ).toHaveBeenCalled();
      expect(
        mockDsqlSigner.prototype.getDbConnectAuthToken
      ).not.toHaveBeenCalled();
    });

    it("should generate regular token for non-admin user", async () => {
      const token = await AuroraDSQLUtil.getDSQLToken(
        "cluster.dsql.us-east-1.on.aws",
        "testuser",
        "default",
        "us-east-1"
      );

      expect(token).toBe(mockToken);
      expect(mockDsqlSigner.prototype.getDbConnectAuthToken).toHaveBeenCalled();
      expect(
        mockDsqlSigner.prototype.getDbConnectAdminAuthToken
      ).not.toHaveBeenCalled();
    });

    it("should use custom credentials provider when provided", async () => {
      const customCredential = {
        accessKeyId: "custom",
        secretAccessKey: "custom",
      };

      await AuroraDSQLUtil.getDSQLToken(
        "cluster.dsql.us-east-1.on.aws",
        "admin",
        "default",
        "us-east-1",
        undefined,
        customCredential as any
      );

      expect(mockFromNodeProviderChain).not.toHaveBeenCalled();
      expect(mockDsqlSigner).toHaveBeenCalledWith(
        expect.objectContaining({
          credentials: customCredential,
        })
      );
    });

    it("should use profile when no custom credentials", async () => {
      await AuroraDSQLUtil.getDSQLToken(
        "cluster.dsql.us-east-1.on.aws",
        "admin",
        "test-profile",
        "us-east-1"
      );

      expect(mockFromNodeProviderChain).toHaveBeenCalledWith({
        profile: "test-profile",
      });
    });

    it("should pass tokenDurationSecs to signer", async () => {
      await AuroraDSQLUtil.getDSQLToken(
        "cluster.dsql.us-east-1.on.aws",
        "admin",
        "default",
        "us-east-1",
        3600
      );

      expect(mockDsqlSigner).toHaveBeenCalledWith(
        expect.objectContaining({
          expiresIn: 3600,
        })
      );
    });

    it("should throw error when token generation fails", async () => {
      mockDsqlSigner.prototype.getDbConnectAdminAuthToken = jest
        .fn()
        .mockRejectedValue(new Error("Auth failed"));

      await expect(
        AuroraDSQLUtil.getDSQLToken(
          "cluster.dsql.us-east-1.on.aws",
          "admin",
          "default",
          "us-east-1"
        )
      ).rejects.toThrow("Failed to generate DSQL token");
    });

    it("should throw error when token is empty", async () => {
      mockDsqlSigner.prototype.getDbConnectAdminAuthToken = jest
        .fn()
        .mockResolvedValue("");

      await expect(
        AuroraDSQLUtil.getDSQLToken(
          "cluster.dsql.us-east-1.on.aws",
          "admin",
          "default",
          "us-east-1"
        )
      ).rejects.toThrow("Failed to retrieve DSQL token - token is empty");
    });
  });

  describe("validatePgConfig", () => {
    it("should validate config with full hostname", () => {
      const config = {
        host: "cluster.dsql.us-east-1.on.aws",
        user: "testuser",
      };

      const result = AuroraDSQLUtil.parsePgConfig(config);

      expect(result).toMatchObject({
        host: "cluster.dsql.us-east-1.on.aws",
        user: "testuser",
        port: 5432,
        database: "postgres",
        region: "us-east-1",
        ssl: { rejectUnauthorized: true },
      });
    });

    it("should parse connection string", () => {
      const connectionString =
        "postgresql://testuser@cluster.dsql.us-east-1.on.aws:5432/testdb";

      const result = AuroraDSQLUtil.parsePgConfig(connectionString);

      expect(result.user).toBe("testuser");
      expect(result.host).toBe("cluster.dsql.us-east-1.on.aws");
    });

    it("should override config values with connectionString values", () => {
      const config = {
        host: "config-host.dsql.us-west-2.on.aws",
        user: "config-user",
        database: "config-db",
        connectionString:
          "postgresql://connstr-user@connstr-host.dsql.us-east-1.on.aws:5432/connstr-db",
      };

      const result = AuroraDSQLUtil.parsePgConfig(config);

      expect(result.host).toBe("connstr-host.dsql.us-east-1.on.aws");
      expect(result.user).toBe("connstr-user");
      expect(result.database).toBe("connstr-db");
      expect(result.region).toBe("us-east-1");
    });

    it("should build hostname from clusterId and region from config", () => {
      const config = {
        host: "cluster123",
        user: "admin",
        region: "us-west-2",
      };

      const result = AuroraDSQLUtil.parsePgConfig(config);

      expect(result.host).toBe("cluster123.dsql.us-west-2.on.aws");
      expect(result.region).toBe("us-west-2");
    });

    // Tolerate unknown endpoint formats for forward compatibility with potential future changes.
    it("should use explicit region when host format is unknown", () => {
      const config = {
        host: "my-cluster.example.com",
        region: "us-east-1",
      };

      const result = AuroraDSQLUtil.parsePgConfig(config);

      expect(result.region).toBe("us-east-1");
      expect(result.host).toBe("my-cluster.example.com");
    });

    it("should respect explicit region override even when hostname is parseable", () => {
      const config = {
        host: "cluster.dsql.us-east-1.on.aws",
        region: "eu-west-1",
      };

      const result = AuroraDSQLUtil.parsePgConfig(config);

      expect(result.region).toBe("eu-west-1");
    });

    it("should use parsed region from hostname even when env vars are set", () => {
      process.env.AWS_REGION = "eu-west-1";
      process.env.AWS_DEFAULT_REGION = "ap-south-1";

      const config = {
        host: "cluster.dsql.us-east-1.on.aws",
      };

      const result = AuroraDSQLUtil.parsePgConfig(config);

      expect(result.region).toBe("us-east-1");
    });

    it("should use AWS_REGION for unknown URL format when region not in config", () => {
      process.env.AWS_REGION = "eu-west-1";

      const config = {
        host: "my-cluster.example.com",
      };

      const result = AuroraDSQLUtil.parsePgConfig(config);

      expect(result.region).toBe("eu-west-1");
      expect(result.host).toBe("my-cluster.example.com");
    });

    it("should use AWS_DEFAULT_REGION for unknown URL format when AWS_REGION not set", () => {
      process.env.AWS_DEFAULT_REGION = "ap-south-1";

      const config = {
        host: "my-cluster.example.com",
      };

      const result = AuroraDSQLUtil.parsePgConfig(config);

      expect(result.region).toBe("ap-south-1");
      expect(result.host).toBe("my-cluster.example.com");
    });

    it("should use AWS_REGION environment variable when region not in config", () => {
      process.env.AWS_REGION = "eu-west-1";

      const config = {
        host: "cluster123",
        user: "admin",
      };

      const result = AuroraDSQLUtil.parsePgConfig(config);

      expect(result.host).toBe("cluster123.dsql.eu-west-1.on.aws");
      expect(result.region).toBe("eu-west-1");
    });

    it("should use AWS_DEFAULT_REGION when AWS_REGION not set", () => {
      process.env.AWS_DEFAULT_REGION = "ap-south-1";

      const config = {
        host: "cluster123",
        user: "admin",
      };

      const result = AuroraDSQLUtil.parsePgConfig(config);

      expect(result.region).toBe("ap-south-1");
    });

    it("should set default user to admin", () => {
      const config = {
        host: "cluster.dsql.us-east-1.on.aws",
      };

      const result = AuroraDSQLUtil.parsePgConfig(config);

      expect(result.user).toBe("admin");
    });

    it("should preserve user-provided values", () => {
      const config = {
        host: "cluster.dsql.us-east-1.on.aws",
        user: "custom",
        port: 3306,
        database: "mydb",
      };

      const result = AuroraDSQLUtil.parsePgConfig(config);

      expect(result.user).toBe("custom");
      expect(result.port).toBe(3306);
      expect(result.database).toBe("mydb");
    });

    it("should throw error when host is missing", () => {
      const config = { user: "admin" };

      expect(() => AuroraDSQLUtil.parsePgConfig(config as any)).toThrow(
        "Host is required"
      );
    });

    it("should throw error when region cannot be determined", () => {
      const config = {
        host: "cluster123",
      };

      expect(() => AuroraDSQLUtil.parsePgConfig(config)).toThrow(
        "Region is not specified for cluster 'cluster123'"
      );
    });

    it("should throw error with hostname when URL format is unknown and region not specified", () => {
      const config = {
        host: "my-cluster.example.com",
      };

      expect(() => AuroraDSQLUtil.parsePgConfig(config)).toThrow(
        "Region is not specified and could not be parsed from hostname: 'my-cluster.example.com'"
      );
    });
  });

  describe("buildHostnameFromIdAndRegion", () => {
    it("should build hostname from clusterId and region", () => {
      const hostname = AuroraDSQLUtil.buildHostnameFromIdAndRegion(
        "cluster123",
        "us-east-1"
      );
      expect(hostname).toBe("cluster123.dsql.us-east-1.on.aws");
    });

    it("should handle different regions", () => {
      expect(
        AuroraDSQLUtil.buildHostnameFromIdAndRegion("abc", "eu-west-1")
      ).toBe("abc.dsql.eu-west-1.on.aws");
      expect(
        AuroraDSQLUtil.buildHostnameFromIdAndRegion("xyz", "ap-south-1")
      ).toBe("xyz.dsql.ap-south-1.on.aws");
    });
  });
});
