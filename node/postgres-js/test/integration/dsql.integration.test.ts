/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import { auroraDSQLPostgres, auroraDSQLWsPostgres } from '../../src/client';
import postgres from "postgres";
import { jest, describe, test, expect } from '@jest/globals';
import { fromNodeProviderChain } from "@aws-sdk/credential-providers";


jest.setTimeout(30000);

async function verifySuccessfulConnection(sql: postgres.Sql<Record<string, postgres.PostgresType> extends {} ? {} : any>) {
    try {
        const result = await sql`SELECT 1 as test_value`;
        expect(result[0].test_value).toBe(1);
    } finally {
        await sql.end();
    }
}

describe('auroraDSQLPostgres DSQL Integration Tests', () => {
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
            ssl: { rejectUnauthorized: false }
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
        const sql = auroraDSQLPostgres({
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
            region: region,
            port: 5432
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

    test('should set default application_name', async () => {
        const sql = auroraDSQLPostgres({
            host: clusterEndpoint,
            username: 'admin',
            region: region,
        });

        try {
            const result = await sql`SELECT current_setting('application_name') as app_name`;
            const appName = result[0].app_name;
            expect(appName).toBeTruthy();
            expect(appName).toMatch(/^aurora-dsql-nodejs-postgresjs\/\d+\.\d+\.\d+/);
            console.log(`Application name: ${appName}`);
        } finally {
            await sql.end();
        }
    });

    test('should set application_name with ORM prefix', async () => {
        const sql = auroraDSQLPostgres({
            host: clusterEndpoint,
            username: 'admin',
            region: region,
            connection: {
                application_name: 'prisma'
            }
        });

        try {
            const result = await sql`SELECT current_setting('application_name') as app_name`;
            const appName = result[0].app_name;
            expect(appName).toBeTruthy();
            expect(appName).toMatch(/^prisma:aurora-dsql-nodejs-postgresjs\/\d+\.\d+\.\d+/);
            console.log(`Application name with ORM prefix: ${appName}`);
        } finally {
            await sql.end();
        }
    });
});

describe('OCC Retry', () => {
    const clusterEndpoint = process.env.CLUSTER_ENDPOINT;
    const region = process.env.REGION;

    test('should retry OCC conflicts with constructor opt-in', async () => {
        const debugCalls: string[] = [];
        const logger = {
            debug: (msg: string) => debugCalls.push(msg),
            error: (msg: string) => console.error(`[occ] ${msg}`),
        };

        const sqlT1: any = auroraDSQLPostgres({
            host: clusterEndpoint,
            username: 'admin',
            region: region,
            retry: { maxRetries: 5 },
            logger,
        });

        const sqlT2: any = auroraDSQLPostgres({
            host: clusterEndpoint,
            username: 'admin',
            region: region,
        });

        try {
            await sqlT1`CREATE TABLE IF NOT EXISTS occ_test_pjs (id INT PRIMARY KEY, v INT)`;
            await sqlT1`INSERT INTO occ_test_pjs (id, v) VALUES (1, 0) ON CONFLICT (id) DO UPDATE SET v = 0`;

            let t1Attempts = 0;
            let t2Done: () => void;
            const t2Finished = new Promise<void>(r => { t2Done = r; });

            const t1 = sqlT1.begin(async (tx: any) => {
                t1Attempts++;
                const [row] = await tx`SELECT v FROM occ_test_pjs WHERE id = 1`;
                if (t1Attempts === 1) await t2Finished;
                await tx`UPDATE occ_test_pjs SET v = ${row.v + 10} WHERE id = 1`;
            });

            await sqlT2.begin(async (tx: any) => {
                await tx`UPDATE occ_test_pjs SET v = v + 1 WHERE id = 1`;
            });
            t2Done!();

            await t1;

            expect(t1Attempts).toBeGreaterThanOrEqual(2);
            expect(debugCalls.some(m => m.includes('OCC conflict'))).toBe(true);
            const finalResult = await sqlT1`SELECT v FROM occ_test_pjs WHERE id = 1`;
            expect(finalResult[0].v).toBe(11);
        } finally {
            await sqlT1`DROP TABLE IF EXISTS occ_test_pjs`;
            await sqlT1.end();
            await sqlT2.end();
        }
    });

    test('should retry OCC conflicts with per-call opt-in', async () => {
        const debugCalls: string[] = [];
        const logger = {
            debug: (msg: string) => debugCalls.push(msg),
            error: (msg: string) => console.error(`[occ] ${msg}`),
        };

        const sql: any = auroraDSQLPostgres({
            host: clusterEndpoint,
            username: 'admin',
            region: region,
            logger,
        });

        try {
            await sql`CREATE TABLE IF NOT EXISTS occ_test_percall (id INT PRIMARY KEY, value INT)`;
            await sql`INSERT INTO occ_test_percall (id, value) VALUES (1, 0) ON CONFLICT (id) DO UPDATE SET value = 0`;

            const updatePromises = Array.from({ length: 3 }, () =>
                sql.begin(async (tx: any) => {
                    const result = await tx`SELECT value FROM occ_test_percall WHERE id = 1`;
                    const currentValue = result[0].value;
                    await tx`UPDATE occ_test_percall SET value = ${currentValue + 1} WHERE id = 1`;
                }, { retry: { maxRetries: 5 } })
            );

            await Promise.all(updatePromises);

            const finalResult = await sql`SELECT value FROM occ_test_percall WHERE id = 1`;
            expect(finalResult[0].value).toBe(3);
            expect(debugCalls.some(m => m.includes('OCC conflict'))).toBe(true);
        } finally {
            await sql`DROP TABLE IF EXISTS occ_test_percall`;
            await sql.end();
        }
    });

    test('should not retry non-OCC errors', async () => {
        const sql: any = auroraDSQLPostgres({
            host: clusterEndpoint,
            username: 'admin',
            region: region,
            retry: { maxRetries: 3 },
        });

        try {
            let attempts = 0;
            await expect(
                sql.begin(async (tx: any) => {
                    attempts++;
                    await tx`SELECT * FROM nonexistent_table_xyz`;
                })
            ).rejects.toThrow();
            expect(attempts).toBe(1);
        } finally {
            await sql.end();
        }
    });
});

// Websocket is not available by default until node 21 and above
const isNode20 = process.version.startsWith('v20.');
(isNode20 ? describe.skip : describe)('auroraDSQLWsPostgres DSQL Integration Tests', () => {
    const clusterEndpoint = process.env.CLUSTER_ENDPOINT;
    const region = process.env.REGION;

    // Testing auroraDSQLWsPostgres 

    test('should connect to DSQL cluster', async () => {
        const sql = auroraDSQLWsPostgres({
            host: clusterEndpoint,
            database: 'postgres',
            username: 'admin',
            region: region
        });
        await verifySuccessfulConnection(sql);
    });

    test('should connect without providing region', async () => {
        const sql = auroraDSQLWsPostgres({
            host: clusterEndpoint,
            database: 'postgres',
            username: 'admin'
        });
        await verifySuccessfulConnection(sql);
    });

    test('should connect without providing database', async () => {
        const sql = auroraDSQLWsPostgres({
            host: clusterEndpoint,
            username: 'admin',
            region: region
        });
        await verifySuccessfulConnection(sql);
    });

    test('should connect with minimum parameters', async () => {
        const sql = auroraDSQLWsPostgres({
            host: clusterEndpoint,
            username: 'admin',
        });
        await verifySuccessfulConnection(sql);
    });

    test('should execute basic query', async () => {
        const sql = auroraDSQLWsPostgres({
            host: clusterEndpoint,
            database: 'postgres',
            username: 'admin',
            region: region,
        });
        await verifySuccessfulConnection(sql);
    });

    test('should handle connection string format', async () => {
        const connectionString = `postgres://admin@${clusterEndpoint}`;

        const sql = auroraDSQLWsPostgres(connectionString);
        await verifySuccessfulConnection(sql);
    });

    test('should handle parameterized queries', async () => {
        const sql = auroraDSQLWsPostgres({
            host: clusterEndpoint,
            database: 'postgres',
            username: 'admin',
            region: region
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
        const sql = auroraDSQLWsPostgres({
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
        const nonAdminSql = auroraDSQLWsPostgres({
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

        const sql = auroraDSQLWsPostgres(connectionString, {
            user: "admin"
        });
        await verifySuccessfulConnection(sql);
    });

    test('should handle clusterID as host', async () => {
        const clusterID = clusterEndpoint!.split(".")[0];
        const sql = auroraDSQLWsPostgres({
            host: clusterID,
            region: region,
            user: "admin"
        });
        await verifySuccessfulConnection(sql);
    });

    test('should handle clusterID as host in connection string', async () => {
        const clusterID = clusterEndpoint!.split(".")[0];
        const connectionString = `postgres://${clusterID}`;

        const sql = auroraDSQLWsPostgres(connectionString, {
            user: "admin",
            region: region
        });
        await verifySuccessfulConnection(sql);
    });


});
