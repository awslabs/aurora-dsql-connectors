/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import { ClientConfig } from "pg";
import { AwsCredentialIdentity, AwsCredentialIdentityProvider } from "@smithy/types";

interface AuroraDSQLConfig extends ClientConfig {
  profile?: string;
  region?: string;
  tokenDurationSecs?: number;
  customCredentialsProvider?: AwsCredentialIdentity | AwsCredentialIdentityProvider;
  logger?: (msg: string) => void;
  transaction?: {
    retry?: RetryConfig;
  };
}

interface RetryConfig {
  maxAttempts?: number;
  baseDelayMs?: number;
  maxDelayMs?: number;
  jitter?: boolean;
}

interface TransactionOptions {
  retry?: RetryConfig | false;
}

export { AuroraDSQLConfig, RetryConfig, TransactionOptions };
