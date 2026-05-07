/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import { Client } from "pg";
import { AuroraDSQLConfig, TransactionOptions } from "./config/aurora-dsql-config.js";
import { AuroraDSQLUtil } from "./aurora-dsql-util.js";
import { resolveRetryConfig, executeTransaction } from "./occ-retry.js";

class AuroraDSQLClient extends Client {
  private dsqlConfig?: AuroraDSQLConfig;

  constructor(config?: string | AuroraDSQLConfig) {
    if (config === undefined) {
      throw new Error("Configuration is required");
    }

    let dsqlConfig = AuroraDSQLUtil.parsePgConfig(config);
    super(dsqlConfig);

    this.dsqlConfig = dsqlConfig;
  }

  // TypeScript doesn't allow multiple declarations of the same function name hence the following declaration was used
  override async connect(callback?: (err: Error) => void) {
    if (this.dsqlConfig !== undefined) {
      try {
        this.password = await AuroraDSQLUtil.getDSQLToken(
          this.dsqlConfig.host!,
          this.dsqlConfig.user!,
          this.dsqlConfig.profile!,
          this.dsqlConfig.region!,
          this.dsqlConfig.tokenDurationSecs,
          this.dsqlConfig.customCredentialsProvider,
        );
      } catch (error) {
        if (callback) {
          callback(error as Error);
          return;
        }
        throw error;
      }
    }
    if (callback) {
      return super.connect(callback);
    }
    return super.connect();
  }

  async transaction<T>(
    callback: (client: this) => Promise<T>,
    options?: TransactionOptions,
  ): Promise<T> {
    const retryConfig = resolveRetryConfig(this.dsqlConfig?.transaction, options);

    return executeTransaction(async () => {
      await this.query("BEGIN");
      try {
        const result = await callback(this);
        await this.query("COMMIT");
        return result;
      } catch (error) {
        try {
          await this.query("ROLLBACK");
        } catch (rollbackError) {
          this.dsqlConfig?.logger?.(`Failed to rollback transaction: ${rollbackError}`);
        }
        throw error;
      }
    }, retryConfig, this.dsqlConfig?.logger);
  }
}

export { AuroraDSQLClient };
