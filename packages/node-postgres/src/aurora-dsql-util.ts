/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import { DsqlSigner } from "@aws-sdk/dsql-signer";
import { fromNodeProviderChain } from "@aws-sdk/credential-providers";
import {
  AwsCredentialIdentity,
  AwsCredentialIdentityProvider,
} from "@smithy/types";
import { AuroraDSQLConfig } from "./config/aurora-dsql-config.js";
import { AuroraDSQLPoolConfig } from "./config/aurora-dsql-pool-config.js";
import { parseIntoClientConfig } from "pg-connection-string";

const ADMIN_USER = "admin";
const PRE_REGION_HOST_PATTERN = ".dsql.";
const POST_REGION_HOST_PATTERN = ".on.aws";

export class AuroraDSQLUtil {

  /**
   * Parse the provided connection string into an equivalent object.
   */
  private static parseConnectionString(connectionString: string): AuroraDSQLConfig {
    const parsed = parseIntoClientConfig(connectionString) as AuroraDSQLConfig;

    // The user is parsed as an empty string if it is not provided. We remove the
    // key instead, to make it easier to work with the result.
    if (!parsed.user) {
      delete parsed.user;
    }

    // Upstream parsing gives strings by default, but we need a number here.
    if (parsed.tokenDurationSecs !== undefined) {
      parsed.tokenDurationSecs = parseInt(parsed.tokenDurationSecs as unknown as string, 10);
    }

    return parsed;
  }

  public static parseRegion(host: string): string {
    if (!host) {
      throw new Error("Hostname is required to parse region");
    }

    const match = host.match(/\.dsql[^.]*\.([^.]+)\.on\.aws$/);
    if (match) {
      return match[1];
    }

    throw new Error(`Unable to parse region from hostname: '${host}'`);
  }

  public static async getDSQLToken(
    host: string,
    user: string,
    profile: string,
    region: string,
    tokenDurationSecs?: number,
    customCredentialsProvider?:
      | AwsCredentialIdentity
      | AwsCredentialIdentityProvider
  ): Promise<string> {
    let token: string;
    try {
      let credential: AwsCredentialIdentity | AwsCredentialIdentityProvider;

      if (customCredentialsProvider !== undefined) {
        credential = customCredentialsProvider;
      } else {
        credential = fromNodeProviderChain({ profile: profile });
      }

      const signer = new DsqlSigner({
        hostname: host,
        credentials: credential,
        region: region,
        expiresIn: tokenDurationSecs,
      });

      if (user === ADMIN_USER) {
        token = await signer.getDbConnectAdminAuthToken();
      } else {
        token = await signer.getDbConnectAuthToken();
      }
    } catch (error) {
      throw new Error(`Failed to generate DSQL token: ${error}`);
    }

    if (!token || token.trim() === "") {
      throw new Error("Failed to retrieve DSQL token - token is empty");
    }
    return token;
  }

  public static parsePgConfig(
    config: string | AuroraDSQLConfig | AuroraDSQLPoolConfig
  ): AuroraDSQLConfig | AuroraDSQLPoolConfig {
    let dsqlConfig: AuroraDSQLConfig | AuroraDSQLPoolConfig;
    if (typeof config === "string") {
      dsqlConfig = AuroraDSQLUtil.parseConnectionString(config) as AuroraDSQLConfig;
    } else if (config.connectionString) {
      // Connection string properties override as set by upstream library.
      dsqlConfig = Object.assign({}, config, AuroraDSQLUtil.parseConnectionString(config.connectionString));
      delete dsqlConfig.connectionString;
    } else {
      dsqlConfig = config;
    }

    if (!dsqlConfig.host) {
      throw new Error("Host is required");
    }

    // If this doesn't look like a URL, we treat it as the cluster ID rather
    // than the full endpoint.
    const isClusterId = !dsqlConfig.host.includes('.');

    let parsedRegion: string | undefined;
    if (!isClusterId) {
      try {
        parsedRegion = AuroraDSQLUtil.parseRegion(dsqlConfig.host);
      } catch {
        // Couldn't parse region from hostname.
      }
    }

    dsqlConfig.region =
      dsqlConfig.region ||
      parsedRegion ||
      process.env.AWS_REGION ||
      process.env.AWS_DEFAULT_REGION;

    if (dsqlConfig.region === undefined) {
      if (isClusterId) {
        throw new Error(`Region is not specified for cluster '${dsqlConfig.host}'`);
      } else {
        throw new Error(`Region is not specified and could not be parsed from hostname: '${dsqlConfig.host}'`);
      }
    }

    if (isClusterId) {
      dsqlConfig.host = AuroraDSQLUtil.buildHostnameFromIdAndRegion(
        dsqlConfig.host,
        dsqlConfig.region
      );
    }

    // Filter out undefined/null values so they don't override defaults.
    const definedConfig = Object.fromEntries(
      Object.entries(dsqlConfig).filter(([, v]) => v !== undefined && v !== null)
    );

    return {
      user: "admin",
      port: 5432,
      database: "postgres",
      ssl: { rejectUnauthorized: true },
      idleTimeoutMillis: 600000, // 10 min
      maxLifetimeSeconds: 3300, // 55 min
      ...definedConfig,
    };
  }

  public static buildHostnameFromIdAndRegion(
    clusterId: string,
    region: string | undefined
  ) {
    return (
      clusterId + PRE_REGION_HOST_PATTERN + region + POST_REGION_HOST_PATTERN
    );
  }
}
