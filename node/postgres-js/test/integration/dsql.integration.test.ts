/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import {auroraDSQLPostgres} from '../../src/client';
import postgres from "postgres";
import {jest} from '@jest/globals';
import {fromNodeProviderChain} from "@aws-sdk/credential-providers";

jest.setTimeout(30000);

async function verifySuccessfulConnection(sql: postgres.Sql<Record<string, postgres.PostgresType> extends {} ? {} : any>) {
    try {
        const result = await sql`SELECT 1 as test_value`;
        expect(result[0].test_value).toBe(1);
    } finally {
        await sql.end();
    }
}

describe('DSQL Integration Tests', () => {
    const clusterEndpoint = process.env.CLUSTER_ENDPOINT;
    const region = process.env.REGION;

    test('should connect to DSQL cluster', async () => {
        const sql = auroraDSQLPostgres({
            host: clusterEndpoint,
            database: 'postgres',
            username: 'admin',
            region: region,
            port: 5432
        });
        await verifySuccessfulConnection(sql);
    });

    test('should connect without providing region', async () => {
        const sql = auroraDSQLPostgres({
            host: clusterEndpoint,
            database: 'postgres',
            username: 'admin',
            port: 5432
        });
        await verifySuccessfulConnection(sql);
    });

    test('should connect without providing database', async () => {
        const sql = auroraDSQLPostgres({
            host: clusterEndpoint,
            username: 'admin',
            region: region,
            port: 5432
        });
        await verifySuccessfulConnection(sql);
    });

    test('should connect with minimum parameters', async () => {
        const sql = auroraDSQLPostgres({
            host: clusterEndpoint,
            username: 'admin',
        });
        await verifySuccessfulConnection(sql);
    });

    test('should execute basic query', async () => {
        const sql = auroraDSQLPostgres({
            host: clusterEndpoint,
            database: 'postgres',
            username: 'admin',
            region: region,
        });
        await verifySuccessfulConnection(sql);
    });

    test('should handle connection string format', async () => {
        const connectionString = `postgres://admin@${clusterEndpoint}`;

        const sql = auroraDSQLPostgres(connectionString);
        await verifySuccessfulConnection(sql);
    });

    test('should handle parameterized queries', async () => {
        const sql = auroraDSQLPostgres({
            host: clusterEndpoint,
            database: 'postgres',
            username: 'admin',
            region: region,
            ssl: {rejectUnauthorized: false}
        });

        try {
            const testValue = 42;
            const result = await sql`SELECT ${testValue} as param_value`;
            expect(result[0].param_value).toBe("42");
        } finally {
            await sql.end();
        }
    });

    test('should handle connection pool with concurrent queries', async () => {
        const sql = auroraDSQLPostgres({
            host: clusterEndpoint,
            database: 'postgres',
            username: 'admin',
            region: region,
            max: 3
        });

        try {
            const promises = [
                sql`SELECT 1 as value`,
                sql`SELECT 2 as value`,
                sql`SELECT 3 as value`
            ];

            const results = await Promise.all(promises);

            expect(results[0][0].value).toBe(1);
            expect(results[1][0].value).toBe(2);
            expect(results[2][0].value).toBe(3);
        } finally {
            await sql.end();
        }
    });

    test('should connect with non-admin user', async () => {
        let username = 'testuser';
        const nonAdminSql = auroraDSQLPostgres({
            host: clusterEndpoint,
            database: 'postgres',
            username: username,
            region: region,
        });

        try {
            const result = await nonAdminSql`SELECT current_user as username`;
            expect(result[0].username).toBe(username);
        } finally {
            await nonAdminSql.end();
        }
    });

    test('should handle url with username in options', async () => {
        const connectionString = `postgres://${clusterEndpoint}`;

        const sql = auroraDSQLPostgres(connectionString, {
            user: "admin"
        });
        await verifySuccessfulConnection(sql);
    });

    test('should handle clusterID as host', async () => {
        const clusterID = clusterEndpoint!.split(".")[0];
        const sql = auroraDSQLPostgres( {
            host: clusterID,
            region: region,
            user: "admin"
        });
        await verifySuccessfulConnection(sql);
    });

    test('should handle clusterID as host in connection string', async () => {
        const clusterID = clusterEndpoint!.split(".")[0];
        const connectionString = `postgres://${clusterID}`;

        const sql = auroraDSQLPostgres(connectionString, {
            user: "admin",
            region: region
        });
        await verifySuccessfulConnection(sql);
    });

    test('should connect with custom credentials provider', async () => {
        let providerCalled = false;
        const trackingProvider = async () => {
            providerCalled = true;
            return fromNodeProviderChain()();
        };

        const sql = auroraDSQLPostgres({
            host: clusterEndpoint,
            username: 'admin',
            customCredentialsProvider: trackingProvider,
        });

        await verifySuccessfulConnection(sql);
        expect(providerCalled).toBe(true);
    });

    test('should connect with custom credentials identity', async () => {
        const credentials = await fromNodeProviderChain()();
        const sql = auroraDSQLPostgres({
            host: clusterEndpoint,
            username: 'admin',
            customCredentialsProvider: credentials,
        });

        await verifySuccessfulConnection(sql);
    });

    // Verifies the provider takes precedence over any other credentials source.
    test('should fail with invalid custom credentials provider', async () => {
        const invalidProvider = async () => ({
            accessKeyId: "INVALID_ACCESS_KEY",
            secretAccessKey: "INVALID_SECRET_KEY",
        });

        const sql = auroraDSQLPostgres({
            host: clusterEndpoint,
            username: 'admin',
            customCredentialsProvider: invalidProvider,
        });

        await expect(sql`SELECT 1`).rejects.toThrow();
    });

    // Verifies the identity takes precedence over any other credentials source.
    test('should fail with invalid custom credentials identity', async () => {
        const sql = auroraDSQLPostgres({
            host: clusterEndpoint,
            username: 'admin',
            customCredentialsProvider: {
                accessKeyId: "INVALID_ACCESS_KEY",
                secretAccessKey: "INVALID_SECRET_KEY",
            },
        });

        await expect(sql`SELECT 1`).rejects.toThrow();
    });
});
