/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import { Pool, PoolClient } from "pg";
import { AuroraDSQLPoolConfig } from "./config/aurora-dsql-pool-config.js";
import { AuroraDSQLUtil } from "./aurora-dsql-util.js";
import { resolveRetryConfig, executeWithRetry } from "./occ-retry.js";

class AuroraDSQLPool extends Pool {
  private dsqlConfig?: AuroraDSQLPoolConfig;

  constructor(config?: AuroraDSQLPoolConfig) {
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

  connect(): Promise<PoolClient>;
  connect(
    callback: (
      err: Error | undefined,
      client: PoolClient | undefined,
      done: (release?: boolean | Error) => void,
    ) => void,
  ): void;

  override async connect(
    callback?: (
      err: Error | undefined,
      client: PoolClient | undefined,
      done: (release?: boolean | Error) => void,
    ) => void,
  ): Promise<PoolClient | void> {
    try {
      if (this.options !== undefined && this.dsqlConfig !== undefined) {
        this.options.password = await AuroraDSQLUtil.getDSQLToken(
          this.dsqlConfig.host!,
          this.dsqlConfig.user!,
          this.dsqlConfig.profile!,
          this.dsqlConfig.region!,
          this.dsqlConfig.tokenDurationSecs,
          this.dsqlConfig.customCredentialsProvider,
        );
      } else {
        throw new Error("options is undefined in this pool");
      }
    } catch (error) {
      if (callback) {
        callback(error as Error, undefined, () => {});
        return;
      }
      throw error;
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
   */
  async transaction<T>(
    callback: (client: PoolClient) => Promise<T>,
    options?: { maxRetries?: number },
  ): Promise<T> {
    const retryConfig = resolveRetryConfig(this.dsqlConfig?.retry, options?.maxRetries);

    return executeWithRetry(async () => {
      const client = await this.connect();
      let destroyConnection = false;
      try {
        await client.query("BEGIN");
        try {
          const result = await callback(client);
          await client.query("COMMIT");
          return result;
        } catch (error) {
          try {
            await client.query("ROLLBACK");
          } catch (rollbackError) {
            this.dsqlConfig?.logger?.error(`Failed to rollback transaction: ${rollbackError}`);
            destroyConnection = true;
          }
          throw error;
        }
      } finally {
        try {
          client.release(destroyConnection);
        } catch (releaseError) {
          this.dsqlConfig?.logger?.error(`Failed to release pool connection: ${releaseError}`);
        }
      }
    }, retryConfig, this.dsqlConfig?.logger);
  }
}

export { AuroraDSQLPool };
