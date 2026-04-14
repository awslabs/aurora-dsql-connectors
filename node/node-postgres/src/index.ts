/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
export { AuroraDSQLClient } from "./aurora-dsql-client.js";
export { AuroraDSQLPool } from "./aurora-dsql-pool.js";
export type { AuroraDSQLConfig } from './config/aurora-dsql-config.js';
export type { AuroraDSQLPoolConfig } from './config/aurora-dsql-pool-config.js';

// OCC Retry exports
export { OCCType, OccRetryExhaustedError, DEFAULT_OCC_CONFIG, validateOccConfig } from './occ-retry.js';
export type { OccRetryConfig, OccErrorInfo, OccRetryEvent, OccRetryExhaustedEvent } from './occ-retry.js';
export type { QueryConfigWithRetry } from './aurora-dsql-client.js';
