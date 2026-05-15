/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import { DatabaseError } from "pg";

interface Logger {
  debug?(msg: string, ...args: unknown[]): void;
  warn(msg: string, ...args: unknown[]): void;
  error(msg: string, ...args: unknown[]): void;
}

interface OCCRetryConfig {
  maxRetries: number;
  baseDelayMs: number;
  maxDelayMs: number;
  jitterFactor: number;
}

const DEFAULT_CONFIG: OCCRetryConfig = {
  maxRetries: 3,
  baseDelayMs: 1,
  maxDelayMs: 100,
  jitterFactor: 0.25,
};

function validateRetryConfig(config: OCCRetryConfig): void {
  for (const [k, v] of Object.entries(config)) {
    if (typeof v !== "number" || !Number.isFinite(v)) {
      throw new Error(`${k} must be a finite number`);
    }
  }
  if (!Number.isInteger(config.maxRetries)) {
    throw new Error("maxRetries must be an integer");
  }
  if (config.maxRetries < 0) {
    throw new Error("maxRetries must be >= 0");
  }
  if (config.maxRetries > 100) {
    throw new Error("maxRetries must not exceed 100");
  }
  if (config.baseDelayMs <= 0) {
    throw new Error("baseDelayMs must be greater than 0");
  }
  if (config.maxDelayMs < config.baseDelayMs) {
    throw new Error("maxDelayMs must be >= baseDelayMs");
  }
  if (config.maxDelayMs > 100) {
    throw new Error("maxDelayMs must not exceed 100");
  }
  if (config.jitterFactor < 0 || config.jitterFactor > 1) {
    throw new Error("jitterFactor must be between 0.0 and 1.0");
  }
}

function resolveRetryConfig(
  constructorRetryConfig: Partial<OCCRetryConfig> | undefined,
  callMaxRetries: number | undefined,
): OCCRetryConfig {
  const resolved: OCCRetryConfig = {
    ...DEFAULT_CONFIG,
    ...constructorRetryConfig,
  };

  if (callMaxRetries !== undefined) {
    resolved.maxRetries = callMaxRetries;
  }

  validateRetryConfig(resolved);
  return resolved;
}

function isOCCError(error: unknown): boolean {
  if (!error || typeof error !== "object") {
    return false;
  }

  const dbError = error as DatabaseError;
  if (!dbError.code) {
    return false;
  }

  return dbError.code === "OC000" || dbError.code === "OC001" || dbError.code === "40001";
}

function calculateBackoff(config: OCCRetryConfig, attempt: number): number {
  const exponent = Math.min(attempt - 1, 31);
  const delay = Math.min(config.baseDelayMs * Math.pow(2.0, exponent), config.maxDelayMs);
  const jitter = delay * Math.random() * config.jitterFactor;
  return delay + jitter;
}

async function executeWithRetry<T>(
  executeCallback: () => Promise<T>,
  config: OCCRetryConfig,
  logger?: Logger,
): Promise<T> {
  if (config.maxRetries <= 0) {
    return executeCallback();
  }

  let lastError: Error | undefined;

  for (let attempt = 1; attempt <= config.maxRetries + 1; attempt++) {
    try {
      return await executeCallback();
    } catch (error) {
      if (!isOCCError(error)) {
        throw error;
      }

      lastError = error as Error;

      if (attempt <= config.maxRetries) {
        const delay = calculateBackoff(config, attempt);
        logger?.debug?.(
          `OCC conflict on attempt ${attempt}/${config.maxRetries + 1}, retrying in ${delay.toFixed(1)}ms`,
        );

        await new Promise<void>((resolve) => setTimeout(resolve, delay));
      }
    }
  }

  logger?.error(`OCC retry exhausted after ${config.maxRetries + 1} attempts`);
  throw lastError!;
}

export { Logger, OCCRetryConfig, resolveRetryConfig, executeWithRetry, isOCCError };

