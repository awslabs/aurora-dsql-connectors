/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import { Client } from "pg";
import { AuroraDSQLConfig } from "./config/aurora-dsql-config.js";
import { AuroraDSQLUtil } from "./aurora-dsql-util.js";
import { resolveRetryConfig, executeWithRetry } from "./occ-retry.js";

class AuroraDSQLClient extends Client {
  private dsqlConfig?: AuroraDSQLConfig;

  constructor(config?: string | AuroraDSQLConfig) {
    if (config === undefined) {
      throw new Error("Configuration is required");
    }

    let dsqlConfig = AuroraDSQLUtil.parsePgConfig(config);
    super(dsqlConfig);

    this.dsqlConfig = dsqlConfig;

    if (dsqlConfig.retry) {
      resolveRetryConfig(dsqlConfig.retry, undefined);
    }
  }

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

  /**
   * Execute a callback within a transaction with automatic OCC retry.
   *
   * The callback may be invoked multiple times on OCC conflicts — it must be
   * idempotent (no side effects that should not be repeated, e.g. sending
   * emails, enqueuing messages, incrementing external counters).
   *
   * Not safe for concurrent calls on the same AuroraDSQLClient instance.
   * Use AuroraDSQLPool.transaction() for concurrent transactional work,
   * where each call acquires its own connection.
   */
  async transaction<T>(
    callback: (client: this) => Promise<T>,
    options?: { maxRetries?: number },
  ): Promise<T> {
    const retryConfig = resolveRetryConfig(this.dsqlConfig?.retry, options?.maxRetries);

    return executeWithRetry(async () => {
      await this.query("BEGIN");
      try {
        const result = await callback(this);
        await this.query("COMMIT");
        return result;
      } catch (error) {
        try {
          await this.query("ROLLBACK");
        } catch (rollbackError) {
          this.dsqlConfig?.logger?.error(`Failed to rollback transaction: ${rollbackError}`);
        }
        throw error;
      }
    }, retryConfig, this.dsqlConfig?.logger);
  }
}

export { AuroraDSQLClient };
