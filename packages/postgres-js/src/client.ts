/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import postgres, {PostgresType} from "postgres";
import {AwsCredentialIdentity, AwsCredentialIdentityProvider} from "@aws-sdk/types";
import {DsqlSigner, DsqlSignerConfig} from "@aws-sdk/dsql-signer";

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
export function auroraDSQLPostgres<T extends Record<string, postgres.PostgresType> = {}>(
    url: string,
    options?: AuroraDSQLConfig<T>
): postgres.Sql<Record<string, postgres.PostgresType> extends T ? {} : {
    [type in keyof T]: T[type] extends {
        serialize: (value: infer R) => any,
        parse: (raw: any) => infer R
    } ? R : never
}>;

export function auroraDSQLPostgres<T extends Record<string, postgres.PostgresType> = {}>(
    options: AuroraDSQLConfig<T>
): postgres.Sql<Record<string, postgres.PostgresType> extends T ? {} : {
    [type in keyof T]: T[type] extends {
        serialize: (value: infer R) => any,
        parse: (raw: any) => infer R
    } ? R : never
}>;

export function auroraDSQLPostgres<T extends Record<string, postgres.PostgresType> = {}>(
    urlOrOptions: string | AuroraDSQLConfig<T>,
    options?: AuroraDSQLConfig<T>
): postgres.Sql<Record<string, postgres.PostgresType> extends T ? {} : {
    [type in keyof T]: T[type] extends {
        serialize: (value: infer R) => any,
        parse: (raw: any) => infer R
    } ? R : never
}> {
/* eslint-enable @typescript-eslint/no-explicit-any */
    let opts: AuroraDSQLConfig<T>;
    let host: string;
    let username: string;
    let database: string | undefined;
    let ssl: object | boolean | string | undefined;
    if (typeof urlOrOptions === 'string') {
        // Called with (url, options)
        let parsedOptions = parseConnectionString(urlOrOptions);
        host = options?.hostname || options?.host || parsedOptions.host || process.env.PGHOST!;
        username = options?.username || options?.user || parsedOptions.username! || process.env.PGUSERNAME || process.env.USER || ADMIN;
        database = options?.database || options?.db || parsedOptions.database || process.env.PGDATABASE;
        ssl = options?.ssl || parsedOptions.ssl;
        opts = {...options!};
    } else {
        // Called with (options) only
        host = urlOrOptions?.hostname || urlOrOptions?.host || process.env.PGHOST!;
        username = urlOrOptions?.username || urlOrOptions?.user || process.env.PGUSERNAME || process.env.USER || ADMIN;
        database = urlOrOptions?.database || urlOrOptions?.db || process.env.PGDATABASE;
        ssl = urlOrOptions?.ssl;
        opts = {...urlOrOptions!};
    }
    if (Array.isArray(host)) {
        throw new Error('Multi-host configurations are not supported for Aurora DSQL');
    }
    let region = opts.region || parseRegionFromHost(host);
    if (isClusterID(host)) {
        host = buildHostnameFromIdAndRegion(host, region);
        opts.host = host;
    }
    let signerConfig: DsqlSignerConfig = {
        hostname: host,
        region: opts.region || parseRegionFromHost(host),
        expiresIn: opts.tokenDurationSecs ?? opts.connect_timeout ?? (process.env.PGCONNECT_TIMEOUT ? parseInt(process.env.PGCONNECT_TIMEOUT) : undefined) ?? DEFAULT_EXPIRY,
        profile: opts.profile || process.env.AWS_PROFILE || "default",
    };
    if (opts.customCredentialsProvider) {
        signerConfig.credentials = opts.customCredentialsProvider;
    }
    let signer = new DsqlSigner(signerConfig);
    if (!database) opts.database = DEFAULT_DATABASE;
    if (!ssl) opts.ssl = true;

    // Build application_name with optional ORM prefix
    const applicationName = buildApplicationName(opts.connection?.application_name);

    const postgresOpts: postgres.Options<T> = {
        ...opts,
        pass: () => getToken(signer, username),
        connection: {
            ...opts.connection,
            application_name: applicationName,
        },
    };
    return typeof urlOrOptions === 'string' ? postgres(urlOrOptions, postgresOpts) : postgres(postgresOpts);
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
    if (parsed.hostname?.includes(',')) {
        throw new Error('Multi-host connection strings are not supported for Aurora DSQL');
    }

    return {
        username: parsed.username,
        host: parsed.hostname,
        database: parsed.pathname?.slice(1),
        ssl: parsed.searchParams.get("ssl") || parsed.searchParams.get("sslmode") || undefined,
    };
}

function parseRegionFromHost(host: string): string | undefined {
    if (!host) {
        throw new Error("Hostname is required to parse region");
    }

    const match = host.match(/^(?<instance>[^.]+)\.(?<dns>dsql(?:-[^.]+)?)\.(?<domain>(?<region>[a-zA-Z0-9-]+)\.on\.aws\.?)$/i);
    if (match?.groups) {
        return match.groups.region;
    }
    throw new Error(`Unable to parse region from hostname: ${host}`);
}

function isClusterID(host: string) {
    return !host.includes(".");
}

function buildHostnameFromIdAndRegion(host: string, region: string | undefined) {
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