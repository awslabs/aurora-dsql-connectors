/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import { Client, QueryResult, QueryResultRow, QueryConfig, QueryArrayConfig, QueryConfigValues, QueryArrayResult, Submittable } from "pg";
import { AuroraDSQLConfig } from "./config/aurora-dsql-config.js";
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

class AuroraDSQLClient extends Client {
  private dsqlConfig?: AuroraDSQLConfig;
  private occConfig?: Required<OccRetryConfig>;

  constructor(config?: string | AuroraDSQLConfig) {
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
    callback: (client: this) => Promise<T>,
    occConfig?: Partial<OccRetryConfig>
  ): Promise<T> {
    // Merge client config, custom config, and always enable retry
    const effectiveConfig: Required<OccRetryConfig> = {
      ...DEFAULT_OCC_CONFIG,
      ...this.occConfig,
      ...occConfig,
      enabled: true
    };

    return executeWithRetry(
      async () => {
        await super.query('BEGIN');
        try {
          const result = await callback(this);
          await super.query('COMMIT');
          return result;
        } catch (error) {
          try {
            await super.query('ROLLBACK');
          } catch (rollbackError) {
            console.debug(
              `Rollback failed: original_error=${error}, rollback_error=${rollbackError}`
            );
          }
          throw error;
        }
      },
      effectiveConfig,
      (event) => this.emitOccEvent(event)
    );
  }
}

export { AuroraDSQLClient };
