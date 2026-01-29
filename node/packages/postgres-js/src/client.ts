/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import postgres, { PostgresType } from "postgres";
import {
  AwsCredentialIdentity,
  AwsCredentialIdentityProvider,
} from "@aws-sdk/types";
import { DsqlSigner, DsqlSignerConfig } from "@aws-sdk/dsql-signer";
import { createPostgresWs } from "./postgres-web-socket";

// Version is injected at build time via tsdown
declare const __VERSION__: string;
const version = typeof __VERSION__ !== "undefined" ? __VERSION__ : "0.0.0";

const ADMIN = "admin";
const DEFAULT_DATABASE = "postgres";
const DEFAULT_EXPIRY = 30; // Based on default Postgres.js connect_timeout
// String components of a DSQL hostname, <Cluster ID>.dsql.<region>.on.aws
const PRE_REGION_HOST_PATTERN = ".dsql.";
const POST_REGION_HOST_PATTERN = ".on.aws";
const APPLICATION_NAME = `aurora-dsql-nodejs-postgresjs/${version}`;

/* eslint-disable @typescript-eslint/no-explicit-any */
function parseConnectionParams(
  urlOrOptions: string | any,
  options?: any
): { opts: any; } {
  let opts: any;
  let host: string;
  let username: string;
  let database: string | undefined;

  if (typeof urlOrOptions === "string") {
    let parsedOptions = parseConnectionString(urlOrOptions);
    host = options?.hostname || options?.host || parsedOptions.host || process.env.PGHOST!;
    username = options?.username || options?.user || parsedOptions.username! ||
      process.env.PGUSERNAME || process.env.USER || ADMIN;
    database = options?.database || options?.db || parsedOptions.database || process.env.PGDATABASE;
    opts = {
      ...options,
      ssl: options?.ssl || parsedOptions.ssl
    };
  } else {
    host = urlOrOptions?.hostname || urlOrOptions?.host || process.env.PGHOST!;
    username = urlOrOptions?.username || urlOrOptions?.user ||
      process.env.PGUSERNAME || process.env.USER || ADMIN;
    database = urlOrOptions?.database || urlOrOptions?.db || process.env.PGDATABASE;
    opts = { ...urlOrOptions };
  }

  if (Array.isArray(host)) {
    throw new Error("Multi-host configurations are not supported for Aurora DSQL");
  }

  if (!opts.region) {
    opts.region = parseRegionFromHost(host);
  }

  if (isClusterID(host)) {
    host = buildHostnameFromIdAndRegion(host, opts.region);
  }

  if (!database) {
    opts.database = DEFAULT_DATABASE;
  }

  opts.host = host;
  opts.username = username;
  opts.connection = {
    ...opts.connection,
    application_name: buildApplicationName(opts.connection?.application_name),
  };

  return { opts };
}

function setupDsqlSigner(opts: any): { signerConfig: DsqlSignerConfig; } {

  let signerConfig: DsqlSignerConfig = {
    hostname: opts.host,
    region: opts.region,
    expiresIn: opts.tokenDurationSecs ?? opts.connect_timeout ??
      (process.env.PGCONNECT_TIMEOUT ? parseInt(process.env.PGCONNECT_TIMEOUT) : undefined) ??
      DEFAULT_EXPIRY,
    profile: opts.profile || process.env.AWS_PROFILE || "default",
  };

  if (opts.customCredentialsProvider) {
    signerConfig.credentials = opts.customCredentialsProvider;
  }

  return { signerConfig };
}

export function auroraDSQLPostgres<
  T extends Record<string, postgres.PostgresType> = {}
>(
  url: string,
  options?: AuroraDSQLConfig<T>
): postgres.Sql<
  Record<string, postgres.PostgresType> extends T
  ? {}
  : {
    [type in keyof T]: T[type] extends {
      serialize: (value: infer R) => any;
      parse: (raw: any) => infer R;
    }
    ? R
    : never;
  }
>;

export function auroraDSQLPostgres<
  T extends Record<string, postgres.PostgresType> = {}
>(
  options: AuroraDSQLConfig<T>
): postgres.Sql<
  Record<string, postgres.PostgresType> extends T
  ? {}
  : {
    [type in keyof T]: T[type] extends {
      serialize: (value: infer R) => any;
      parse: (raw: any) => infer R;
    }
    ? R
    : never;
  }
>;

export function auroraDSQLPostgres<
  T extends Record<string, postgres.PostgresType> = {}
>(
  urlOrOptions: string | AuroraDSQLConfig<T>,
  options?: AuroraDSQLConfig<T>
): postgres.Sql<
  Record<string, postgres.PostgresType> extends T
  ? {}
  : {
    [type in keyof T]: T[type] extends {
      serialize: (value: infer R) => any;
      parse: (raw: any) => infer R;
    }
    ? R
    : never;
  }
> {

  let { opts } = parseConnectionParams(urlOrOptions, options);
  const { signerConfig } = setupDsqlSigner(opts);
  let signer = new DsqlSigner(signerConfig);

  if (!opts.ssl) opts.ssl = true;
  const postgresOpts: postgres.Options<T> = {
    ...opts,
    pass: () => getToken(signer, opts.username),
  };
  return typeof urlOrOptions === "string"
    ? postgres(urlOrOptions, postgresOpts)
    : postgres(postgresOpts);
}

export function auroraDSQLWsPostgres<
  T extends Record<string, postgres.PostgresType> = {}
>(
  url: string,
  options?: AuroraDSQLWsConfig<T>
): postgres.Sql<
  Record<string, postgres.PostgresType> extends T
  ? {}
  : {
    [type in keyof T]: T[type] extends {
      serialize: (value: infer R) => any;
      parse: (raw: any) => infer R;
    }
    ? R
    : never;
  }
>;

export function auroraDSQLWsPostgres<
  T extends Record<string, postgres.PostgresType> = {}
>(
  options: AuroraDSQLWsConfig<T>
): postgres.Sql<
  Record<string, postgres.PostgresType> extends T
  ? {}
  : {
    [type in keyof T]: T[type] extends {
      serialize: (value: infer R) => any;
      parse: (raw: any) => infer R;
    }
    ? R
    : never;
  }
>;

export function auroraDSQLWsPostgres<
  T extends Record<string, postgres.PostgresType> = {}
>(
  urlOrOptions: string | AuroraDSQLWsConfig<T>,
  options?: AuroraDSQLWsConfig<T>
): postgres.Sql<
  Record<string, postgres.PostgresType> extends T
  ? {}
  : {
    [type in keyof T]: T[type] extends {
      serialize: (value: infer R) => any;
      parse: (raw: any) => infer R;
    }
    ? R
    : never;
  }
> {

  let { opts } = parseConnectionParams(urlOrOptions, options);
  const { signerConfig } = setupDsqlSigner(opts);

  if (!opts.pass) {
    opts.pass = async () => {
      let signer = new DsqlSigner(signerConfig);
      return await getToken(signer, opts.username);
    };
  }

  // swap out socket to use websocket
  opts.socket = createPostgresWs(opts);

  // ssl must be false otherwise postgres.js will try to use the net.socket 
  opts.ssl = false;

  if (opts.connectionCheck == undefined) {
    // disable connection check by default 
    // connection check sends a 'select 1' before every query
    opts.connectionCheck = false;
  }

  opts.port = 443;

  return typeof urlOrOptions === "string"
    ? postgres(urlOrOptions, opts)
    : postgres(opts);
}


function parseConnectionString(url: string): {
  database?: string;
  host?: string;
  username?: string;
  ssl?: string;
} {
  let decodedUrl = decodeURI(url);
  const parsed = new URL(decodedUrl);

  // Check for multi-host
  if (parsed.hostname?.includes(",")) {
    throw new Error(
      "Multi-host connection strings are not supported for Aurora DSQL"
    );
  }

  return {
    username: parsed.username,
    host: parsed.hostname,
    database: parsed.pathname?.slice(1),
    ssl:
      parsed.searchParams.get("ssl") ||
      parsed.searchParams.get("sslmode") ||
      undefined,
  };
}

function parseRegionFromHost(host: string): string | undefined {
  if (!host) {
    throw new Error("Hostname is required to parse region");
  }

  const match = host.match(
    /^(?<instance>[^.]+)\.(?<dns>dsql(?:-[^.]+)?)\.(?<domain>(?<region>[a-zA-Z0-9-]+)\.on\.aws\.?)$/i
  );
  if (match?.groups) {
    return match.groups.region;
  }
  throw new Error(`Unable to parse region from hostname: ${host}`);
}

function isClusterID(host: string) {
  return !host.includes(".");
}

function buildHostnameFromIdAndRegion(
  host: string,
  region: string | undefined
) {
  return host + PRE_REGION_HOST_PATTERN + region + POST_REGION_HOST_PATTERN;
}

async function getToken(signer: DsqlSigner, username: string): Promise<string> {
  if (username === ADMIN) {
    return await signer.getDbConnectAdminAuthToken();
  } else {
    return await signer.getDbConnectAuthToken();
  }
}

/**
 * Build the application_name with optional ORM prefix.
 *
 * If ormPrefix is provided and non-empty after trimming, prepends it to
 * the connector identifier. Otherwise, returns the connector's application_name.
 *
 * PostgreSQL limits application_name to 64 characters. After accounting for
 * the connector identifier and separator, 27 characters are available for
 * the ORM name.
 *
 * @param ormPrefix Optional ORM name to prepend (e.g., "prisma")
 * @returns Formatted application_name string
 */
function buildApplicationName(ormPrefix?: string): string {
  if (ormPrefix) {
    const trimmed = ormPrefix.trim();
    if (trimmed) {
      return `${trimmed}:${APPLICATION_NAME}`;
    }
  }
  return APPLICATION_NAME;
}

export interface AuroraDSQLConfig<T extends Record<string, PostgresType<T>>> extends Omit<postgres.Options<T>, 'password' | 'pass'> {

  region?: string;

  profile?: string;

  tokenDurationSecs?: number;

  customCredentialsProvider?: AwsCredentialIdentity | AwsCredentialIdentityProvider;

}
export interface AuroraDSQLWsConfig<T extends Record<string, PostgresType<T>>>
  extends Omit<postgres.Options<T>, "socket" | "ssl" | "port"> {
  region?: string;
  tokenDurationSecs?: number;
  customCredentialsProvider?:
  | AwsCredentialIdentity
  | AwsCredentialIdentityProvider;
  connectionCheck?: boolean;
  connectionId?: string;
  onReservedConnectionClose?: (connectionId?: string) => void;
}
