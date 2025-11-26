/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import { PoolConfig } from "pg";
import { AuroraDSQLConfig } from "./aurora-dsql-config.js";

type AuroraDSQLPoolConfig = AuroraDSQLConfig & PoolConfig;

export { AuroraDSQLPoolConfig };
