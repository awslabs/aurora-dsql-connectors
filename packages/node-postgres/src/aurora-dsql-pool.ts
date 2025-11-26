/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import { Pool, PoolClient } from "pg";
import { AuroraDSQLPoolConfig } from "./config/aurora-dsql-pool-config.js";
import { AuroraDSQLUtil } from "./aurora-dsql-util.js";

class AuroraDSQLPool extends Pool {
  private dsqlConfig?: AuroraDSQLPoolConfig;

  constructor(config?: AuroraDSQLPoolConfig) {
    if (config === undefined) {
      throw new Error("Configuration is required");
    }

    let dsqlConfig = AuroraDSQLUtil.parsePgConfig(config);
    super(dsqlConfig);

    this.dsqlConfig = dsqlConfig;
  }

  // These declaration are needed as they have different returns otherwise a compile error will occur
  connect(): Promise<PoolClient>;
  connect(
    callback: (
      err: Error | undefined,
      client: PoolClient | undefined,
      done: (release?: boolean | Error) => void
    ) => void
  ): void;

  // TypeScript doesn't allow multiple declaration of the same name hence the following declaration was used
  override async connect(
    callback?: (
      err: Error | undefined,
      client: PoolClient | undefined,
      done: (release?: boolean | Error) => void
    ) => void
  ): Promise<PoolClient | void> {
    try {
      if (this.options !== undefined && this.dsqlConfig !== undefined) {
        this.options.password = await AuroraDSQLUtil.getDSQLToken(
          this.dsqlConfig.host!,
          this.dsqlConfig.user!,
          this.dsqlConfig.profile!,
          this.dsqlConfig.region!,
          this.dsqlConfig.tokenDurationSecs
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
}

export { AuroraDSQLPool };
