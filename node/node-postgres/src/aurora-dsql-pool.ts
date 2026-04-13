/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import { Pool, PoolClient, QueryResult, QueryResultRow, QueryConfig, QueryArrayConfig, QueryConfigValues, QueryArrayResult, Submittable } from "pg";
import { AuroraDSQLPoolConfig } from "./config/aurora-dsql-pool-config.js";
import { AuroraDSQLUtil } from "./aurora-dsql-util.js";
import {
  DEFAULT_OCC_CONFIG,
  OccRetryConfig,
  executeWithRetry,
  OccRetryEvent,
  OccRetryExhaustedEvent
} from "./occ-retry.js";

// Extended QueryConfig to support skipRetry option
interface QueryConfigWithRetry<I = any[]> extends QueryConfig<I> {
  skipRetry?: boolean;
}

class AuroraDSQLPool extends Pool {
  private dsqlConfig?: AuroraDSQLPoolConfig;
  private occConfig?: Required<OccRetryConfig>;

  constructor(config?: AuroraDSQLPoolConfig) {
    if (config === undefined) {
      throw new Error("Configuration is required");
    }

    let dsqlConfig = AuroraDSQLUtil.parsePgConfig(config);
    super(dsqlConfig);

    this.dsqlConfig = dsqlConfig;

    // Initialize OCC retry config if provided
    if (dsqlConfig.occ) {
      this.occConfig = { ...DEFAULT_OCC_CONFIG, ...dsqlConfig.occ };
    }
  }

  // These declaration are needed as they have different returns otherwise a compile error will occur
  connect(): Promise<PoolClient>;
  connect(
    callback: (
      err: Error | undefined,
      client: PoolClient | undefined,
      done: (release?: boolean | Error) => void,
    ) => void,
  ): void;

  // TypeScript doesn't allow multiple declaration of the same name hence the following declaration was used
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
        callback(error as Error, undefined, () => { });
        return;
      }
      throw error;
    }
    if (callback) {
      return super.connect(callback);
    }
    return super.connect();
  }

  // Override query method with all pg overloads to support OCC retry
  override query<T extends Submittable>(queryStream: T): T;
  override query<R extends any[] = any[], I = any[]>(
    queryConfig: QueryArrayConfig<I>,
    values?: QueryConfigValues<I>,
  ): Promise<QueryArrayResult<R>>;
  override query<R extends QueryResultRow = any, I = any[]>(
    queryConfig: QueryConfigWithRetry<I>,
  ): Promise<QueryResult<R>>;
  override query<R extends QueryResultRow = any, I = any[]>(
    queryTextOrConfig: string | QueryConfigWithRetry<I>,
    values?: QueryConfigValues<I>,
  ): Promise<QueryResult<R>>;
  override query<R extends any[] = any[], I = any[]>(
    queryConfig: QueryArrayConfig<I>,
    callback: (err: Error, result: QueryArrayResult<R>) => void,
  ): void;
  override query<R extends QueryResultRow = any, I = any[]>(
    queryTextOrConfig: string | QueryConfigWithRetry<I>,
    callback: (err: Error, result: QueryResult<R>) => void,
  ): void;
  override query<R extends QueryResultRow = any, I = any[]>(
    queryText: string,
    values: QueryConfigValues<I>,
    callback: (err: Error, result: QueryResult<R>) => void,
  ): void;

  // Implementation
  override query(
    queryTextOrConfig: any,
    valuesOrCallback?: any,
    callback?: any,
  ): any {
    // Pass through for QueryStream (Submittable)
    if (queryTextOrConfig && typeof queryTextOrConfig.submit === 'function') {
      return super.query(queryTextOrConfig, valuesOrCallback, callback);
    }

    // Check if retry should be skipped
    const skipRetry = !this.occConfig?.enabled ||
      (typeof queryTextOrConfig === 'object' && queryTextOrConfig?.skipRetry);

    if (skipRetry) {
      return super.query(queryTextOrConfig, valuesOrCallback, callback);
    }

    // Extract query text for logging
    const queryText = typeof queryTextOrConfig === 'string'
      ? queryTextOrConfig
      : queryTextOrConfig?.text;

    // Determine if callback-based or promise-based
    const hasCallback = typeof valuesOrCallback === 'function' || typeof callback === 'function';
    const actualCallback = (typeof valuesOrCallback === 'function' ? valuesOrCallback : callback);

    // Wrap query with retry logic
    const operation = () => super.query(
      queryTextOrConfig,
      typeof valuesOrCallback === 'function' ? undefined : valuesOrCallback
    );

    if (hasCallback && actualCallback) {
      executeWithRetry(
        operation,
        this.occConfig!,
        (event) => this.emitOccEvent(event),
        queryText
      )
        .then(result => actualCallback(null, result))
        .catch(err => actualCallback(err, undefined as any));
      return;
    }

    return executeWithRetry(
      operation,
      this.occConfig!,
      (event) => this.emitOccEvent(event),
      queryText
    );
  }

  // Emit OCC events (occRetry and occRetryExhausted)
  private emitOccEvent(event: OccRetryEvent | OccRetryExhaustedEvent): void {
    if (event.type === 'occRetry') {
      this.emit('occRetry', event);
    } else {
      this.emit('occRetryExhausted', event);
    }
  }

  // Execute transaction with automatic OCC retry
  async transactionWithRetry<T>(
    callback: (client: PoolClient) => Promise<T>,
    occConfig?: Partial<OccRetryConfig>
  ): Promise<T> {
    // Merge pool config, custom config, and always enable retry
    const effectiveConfig: Required<OccRetryConfig> = {
      ...DEFAULT_OCC_CONFIG,
      ...this.occConfig,
      ...occConfig,
      enabled: true
    };

    return executeWithRetry(
      async () => {
        const client = await this.connect();
        try {
          await client.query('BEGIN');
          const result = await callback(client);
          await client.query('COMMIT');
          return result;
        } catch (error) {
          try {
            await client.query('ROLLBACK');
          } catch (rollbackError) {
            console.debug(
              `Rollback failed: original_error=${error}, rollback_error=${rollbackError}`
            );
          }
          throw error;
        } finally {
          try {
            client.release();
          } catch (releaseError) {
            console.debug(`Client release failed: ${releaseError}`);
          }
        }
      },
      effectiveConfig,
      (event) => this.emitOccEvent(event)
    );
  }
}

export { AuroraDSQLPool };
