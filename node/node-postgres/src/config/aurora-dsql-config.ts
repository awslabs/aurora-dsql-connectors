/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import { ClientConfig } from "pg";
import { AwsCredentialIdentity, AwsCredentialIdentityProvider } from "@smithy/types";
import { OccRetryConfig } from "../occ-retry.js";

interface AuroraDSQLConfig extends ClientConfig {
  profile?: string;
  region?: string;
  tokenDurationSecs?: number;
  customCredentialsProvider?: AwsCredentialIdentity | AwsCredentialIdentityProvider;
  occ?: OccRetryConfig;
}

export { AuroraDSQLConfig };
