/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import { DatabaseError } from "pg";
import { RetryConfig, TransactionOptions } from "./config/aurora-dsql-config.js";

type OCCType = "Data" | "Schema" | "Unknown";

const DEFAULT_RETRY_CONFIG: Required<RetryConfig> = {
  maxAttempts: 3,
  baseDelayMs: 1,
  maxDelayMs: 100,
  jitter: true,
};

function getOCCType(error: unknown): OCCType | null {
  if (!error || typeof error !== "object") {
    return null;
  }

  const dbError = error as DatabaseError;
  if (!dbError.code) {
    return null;
  }

  if (dbError.code === "OC000") {
    return "Data";
  }
  if (dbError.code === "OC001") {
    return "Schema";
  }
  if (dbError.code === "40001") {
    if (dbError.message && (dbError.message.includes("(OC000)") || dbError.message.includes("(OC001)"))) {
      return dbError.message.includes("(OC000)") ? "Data" : "Schema";
    }
    return "Unknown";
  }

  return null;
}

function calculateBackoff(baseDelayMs: number, maxDelayMs: number, jitter: boolean): number {
  if (!jitter) {
    return baseDelayMs;
  }

  const delay = baseDelayMs + Math.random() * baseDelayMs;
  return Math.min(delay, maxDelayMs);
}

function validateRetryConfig(config: Required<RetryConfig>): void {
  if (!Number.isInteger(config.maxAttempts) || config.maxAttempts <= 0) {
    throw new TypeError('maxAttempts must be a positive integer');
  }
  if (!Number.isFinite(config.baseDelayMs) || config.baseDelayMs <= 0) {
    throw new TypeError('baseDelayMs must be a positive number');
  }
  if (!Number.isFinite(config.maxDelayMs) || config.maxDelayMs <= 0) {
    throw new TypeError('maxDelayMs must be a positive number');
  }
  if (config.maxDelayMs < config.baseDelayMs) {
    throw new TypeError('maxDelayMs must be >= baseDelayMs');
  }
}

function resolveRetryConfig(
  constructorConfig: { retry?: RetryConfig } | undefined,
  callOptions: TransactionOptions | undefined,
): Required<RetryConfig> | null {
  if (callOptions?.retry === false) {
    return null;
  }

  const config = {
    ...DEFAULT_RETRY_CONFIG,
    ...(constructorConfig?.retry || {}),
    ...(callOptions?.retry || {}),
  };

  validateRetryConfig(config);

  return config;
}

async function executeTransaction<T>(
  executeCallback: () => Promise<T>,
  retryConfig: Required<RetryConfig> | null,
  logger?: (msg: string) => void,
): Promise<T> {
  if (retryConfig === null) {
    return executeCallback();
  }

  let lastError: Error | undefined;

  for (let attempt = 1; attempt <= retryConfig.maxAttempts; attempt++) {
    try {
      return await executeCallback();
    } catch (error) {
      const occType = getOCCType(error);

      if (occType === null) {
        throw error;
      }

      lastError = error as Error;

      if (attempt < retryConfig.maxAttempts) {
        const delay = calculateBackoff(retryConfig.baseDelayMs, retryConfig.maxDelayMs, retryConfig.jitter);
        logger?.(`OCC conflict (${occType}) on attempt ${attempt}, retrying in ${delay.toFixed(1)}ms`);
        await new Promise((resolve) => setTimeout(resolve, delay));
      } else {
        logger?.(`OCC retry exhausted after ${retryConfig.maxAttempts} attempts (${occType} conflict)`);
        throw error;
      }
    }
  }

  throw lastError;
}

export { resolveRetryConfig, executeTransaction };
