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
import { parse } from "pg-connection-string";

const ADMIN_USER = "admin";
const PRE_REGION_HOST_PATTERN = ".dsql.";
const POST_REGION_HOST_PATTERN = ".on.aws";

export class AuroraDSQLUtil {
  public static parseRegion(host: string): string {
    if (!host) {
      throw new Error("Hostname is required to parse region");
    }

    const match = host.match(/\.dsql[^.]*\.([^.]+)\.on\.aws$/);
    if (match) {
      return match[1];
    }

    throw new Error(`Unable to parse region from hostname: ${host}`);
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
      dsqlConfig = parse(config) as AuroraDSQLConfig;
    } else {
      dsqlConfig = config;
    }

    if (!dsqlConfig.host) {
      throw new Error("Host is required");
    }

    // check if host is a clusterId or cluster endpoint
    try {
      dsqlConfig.region = AuroraDSQLUtil.parseRegion(dsqlConfig.host);
    } catch {
      //clusterId is specified in the host name
      dsqlConfig.region =
        dsqlConfig.region ||
        process.env.AWS_REGION ||
        process.env.AWS_DEFAULT_REGION;
      if (dsqlConfig.region === undefined) {
        throw new Error("Region is not specified");
      }
      dsqlConfig.host = AuroraDSQLUtil.buildHostnameFromIdAndRegion(
        dsqlConfig.host!,
        dsqlConfig.region
      );
    }

    dsqlConfig = {
      user: "admin",
      port: 5432,
      database: "postgres",
      ssl: { rejectUnauthorized: true },
      idleTimeoutMillis: 600000, // 10 min
      maxLifetimeSeconds: 3300, // 55 min
      ...dsqlConfig,
    };
    return dsqlConfig;
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
