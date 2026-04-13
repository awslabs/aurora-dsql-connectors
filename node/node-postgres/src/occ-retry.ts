/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/**
 * OCC error type classification
 */
export enum OCCType {
  Data = 'Data',       // OC000 - concurrent data modification
  Schema = 'Schema',   // OC001 - DDL during transaction
  Unknown = 'Unknown'  // 40001 - generic serialization failure
}

/**
 * OCC error detection result
 */
export interface OccErrorInfo {
  type: OCCType;
  code: string;
}

/**
 * OCC retry configuration
 */
export interface OccRetryConfig {
  enabled: boolean;          // Required to enable retry
  maxAttempts?: number;      // Default: 3, Range: 1-100
  baseDelayMs?: number;      // Default: 1
  maxDelayMs?: number;       // Default: 100
  jitterFactor?: number;     // Default: 0.25, Range: 0-1
}

/**
 * Event emitted on retry attempt
 */
export interface OccRetryEvent {
  type: 'occRetry';
  attempt: number;           // Current attempt number (1-based)
  maxAttempts: number;       // Maximum attempts configured
  delayMs: number;           // Calculated backoff delay
  error: Error;              // Original database error
  occType: OCCType;          // Classified OCC type
  occCode: string;           // SQL error code
  queryText?: string;        // Query text if available
}

/**
 * Event emitted when retries exhausted
 */
export interface OccRetryExhaustedEvent {
  type: 'occRetryExhausted';
  attempts: number;          // Total attempts made
  error: Error;              // Last database error
  occType: OCCType;          // Classified OCC type
  occCode: string;           // SQL error code
  queryText?: string;        // Query text if available
}

/**
 * Custom error thrown when retries exhausted
 */
export class OccRetryExhaustedError extends Error {
  name = 'OccRetryExhaustedError';

  constructor(
    public readonly attempts: number,
    public readonly lastError: Error,
    public readonly occInfo: OccErrorInfo
  ) {
    super(
      `OCC retry exhausted after ${attempts} attempts ` +
      `(type=${occInfo.type}, code=${occInfo.code})`
    );

    // Restore prototype chain for instanceof checks
    Object.setPrototypeOf(this, OccRetryExhaustedError.prototype);
  }
}

/**
 * Default OCC retry configuration
 */
export const DEFAULT_OCC_CONFIG: Required<OccRetryConfig> = {
  enabled: false,
  maxAttempts: 3,
  baseDelayMs: 1,
  maxDelayMs: 100,
  jitterFactor: 0.25
};

/**
 * Detect if error is an OCC error and classify it
 *
 * @param error - Database error to check
 * @returns OccErrorInfo if OCC error, null otherwise
 */
export function isOccError(error: Error): OccErrorInfo | null {
  const dbError = error as any;

  // Check for PostgreSQL error code
  if (!dbError.code) {
    return null;
  }

  // OC000 - Data conflict (concurrent data modification)
  if (dbError.code === 'OC000') {
    return { type: OCCType.Data, code: 'OC000' };
  }

  // OC001 - Schema conflict (DDL during transaction)
  if (dbError.code === 'OC001') {
    return { type: OCCType.Schema, code: 'OC001' };
  }

  // 40001 - Serialization failure (may contain embedded OC000/OC001)
  if (dbError.code === '40001') {
    const message = dbError.message || '';

    // Parse message for embedded OCC codes
    if (message.includes('OC000')) {
      return { type: OCCType.Data, code: 'OC000' };
    }

    if (message.includes('OC001')) {
      return { type: OCCType.Schema, code: 'OC001' };
    }

    // Generic serialization failure
    return { type: OCCType.Unknown, code: '40001' };
  }

  return null;
}

/**
 * Calculate exponential backoff delay with jitter
 *
 * Formula: delay = min(baseDelayMs * 2^(attempt-1) + jitter, maxDelayMs)
 * Jitter: random(0, delay * jitterFactor)
 *
 * @param config - Retry configuration
 * @param attempt - Current attempt number (1-based)
 * @returns Calculated delay in milliseconds
 */
export function calculateBackoff(
  config: Required<OccRetryConfig>,
  attempt: number
): number {
  // Exponential backoff: baseDelayMs * 2^(attempt-1)
  const exponentialDelay = config.baseDelayMs * Math.pow(2, attempt - 1);

  // Calculate jitter: random value between 0 and (delay * jitterFactor)
  const jitterRange = exponentialDelay * config.jitterFactor;
  const jitter = Math.random() * jitterRange;

  // Apply jitter and cap at maxDelayMs
  const delayWithJitter = exponentialDelay + jitter;
  return Math.min(Math.round(delayWithJitter), config.maxDelayMs);
}

/**
 * Sleep for specified milliseconds
 *
 * @param ms - Milliseconds to sleep
 */
export function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * Execute operation with automatic OCC retry
 *
 * @param operation - Operation to execute
 * @param config - Retry configuration
 * @param emitEvent - Event emitter callback
 * @param queryText - Optional query text for logging
 * @returns Operation result
 * @throws OccRetryExhaustedError if retries exhausted
 * @throws Error if non-OCC error occurs
 */
export async function executeWithRetry<T>(
  operation: () => Promise<T>,
  config: Required<OccRetryConfig>,
  emitEvent: (event: OccRetryEvent | OccRetryExhaustedEvent) => void,
  queryText?: string
): Promise<T> {
  let attempt = 1;

  while (true) {
    try {
      return await operation();
    } catch (error) {
      // Detect OCC error
      const occInfo = isOccError(error as Error);

      // Only retry OCC errors (40001, OC000, OC001)
      if (!occInfo) {
        throw error;
      }

      // Check if max attempts reached
      if (attempt >= config.maxAttempts) {
        console.error(
          `OCC transaction retry exhausted, type=${occInfo.type}, code=${occInfo.code}, attempts=${config.maxAttempts}`
        );

        emitEvent({
          type: 'occRetryExhausted',
          attempts: config.maxAttempts,
          error: error as Error,
          occType: occInfo.type,
          occCode: occInfo.code,
          queryText
        });

        throw new OccRetryExhaustedError(config.maxAttempts, error as Error, occInfo);
      }

      // Calculate exponential backoff with jitter
      const delayMs = calculateBackoff(config, attempt);

      console.debug(
        `OCC conflict detected, type=${occInfo.type}, code=${occInfo.code}, ` +
        `retrying after backoff, attempt=${attempt + 1}/${config.maxAttempts}, delay_ms=${delayMs}`
      );

      emitEvent({
        type: 'occRetry',
        attempt: attempt + 1,
        maxAttempts: config.maxAttempts,
        delayMs,
        error: error as Error,
        occType: occInfo.type,
        occCode: occInfo.code,
        queryText
      });

      await sleep(delayMs);
      attempt++;
    }
  }
}
