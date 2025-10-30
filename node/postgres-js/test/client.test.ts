/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import {jest} from '@jest/globals';
import {auroraDSQLPostgres} from "../src";

jest.mock('postgres', () => {
    const mockPostgres = jest.fn(() => ({end: jest.fn()}));
    return {
        default: mockPostgres,
        __esModule: true
    };
});

jest.mock('@aws-sdk/dsql-signer', () => {
    const mockGetDbConnectAdminAuthToken = jest.fn<() => Promise<string>>().mockResolvedValue('admin-token');
    const mockGetDbConnectAuthToken = jest.fn<() => Promise<string>>().mockResolvedValue('user-token');
    const mockDsqlSigner = jest.fn().mockImplementation(() => ({
        getDbConnectAdminAuthToken: mockGetDbConnectAdminAuthToken,
        getDbConnectAuthToken: mockGetDbConnectAuthToken
    }));

    return {
        DsqlSigner: mockDsqlSigner,
        __esModule: true
    };
});

describe('AuroraDSQLPostgres', () => {
    let AuroraDSQLPostgres: any;
    let mockPostgres: any;
    let mockDsqlSigner: any;

    beforeAll(async () => {
        const postgresModule = await import('postgres');
        const dsqlModule = await import('@aws-sdk/dsql-signer');

        mockPostgres = postgresModule.default;
        mockDsqlSigner = dsqlModule.DsqlSigner;

        // Import after mocking
        const module = await import('../src/client');
        AuroraDSQLPostgres = module.auroraDSQLPostgres;
    });

    beforeEach(() => {
        jest.clearAllMocks();
    });

    describe('parseUrl functionality', () => {
        test('should parse connection string with username and host', async () => {
            AuroraDSQLPostgres('postgres://admin@cluster.dsql.us-east-1.on.aws/postgres', {
                region: 'us-east-1'
            });

            expect(mockPostgres).toHaveBeenCalledTimes(1);
            const [url, options] = mockPostgres.mock.calls[0];
            expect(url).toBe('postgres://admin@cluster.dsql.us-east-1.on.aws/postgres');
            expect(typeof options.pass).toBe('function');
            expect(mockDsqlSigner).toHaveBeenCalledTimes(1);
            const signerConfig = mockDsqlSigner.mock.calls[0][0];
            expect(signerConfig.hostname).toBe('cluster.dsql.us-east-1.on.aws');
            const token = await options.pass();
            expect(token).toBe('admin-token');
        });

        test('should parse connection string without explicit region', () => {
            AuroraDSQLPostgres('postgres://testuser@cluster.dsql.us-west-2.on.aws/testdb');

            expect(mockDsqlSigner).toHaveBeenCalledTimes(1);
            const signerConfig = mockDsqlSigner.mock.calls[0][0];
            expect(signerConfig.region).toBe('us-west-2');
            expect(signerConfig.hostname).toBe('cluster.dsql.us-west-2.on.aws');
        });

        test('should parse database from connection string pathname', () => {
            AuroraDSQLPostgres('postgres://admin@cluster.dsql.us-east-1.on.aws/mydb', {
                region: 'us-east-1'
            });

            expect(mockPostgres).toHaveBeenCalledTimes(1);
            const [url, options] = mockPostgres.mock.calls[0];
            expect(url).toBe('postgres://admin@cluster.dsql.us-east-1.on.aws/mydb');
            expect(options.database).toBeUndefined();
        });

        test('should handle connection string without database', () => {
            AuroraDSQLPostgres('postgres://admin@cluster.dsql.us-east-1.on.aws/', {
                region: 'us-east-1'
            });

            expect(mockPostgres).toHaveBeenCalledTimes(1);
            const [url, options] = mockPostgres.mock.calls[0];
            expect(url).toBe('postgres://admin@cluster.dsql.us-east-1.on.aws/');
            expect(options.database).toBe('postgres');
        });

        test('should throw error on multi host with username', () => {
            expect(() => {
                auroraDSQLPostgres('postgres://admin@host1,host2/postgres');
            }).toThrow('Multi-host connection strings are not supported for Aurora DSQL');
        });

        test('should throw error on multi host', () => {
            expect(() => {
                auroraDSQLPostgres('postgres://host1,host2/postgres');
            }).toThrow('Multi-host connection strings are not supported for Aurora DSQL');
        });
    });

    describe('parseRegionFromHost', () => {
        test('should extract region from DSQL hostname', () => {
            AuroraDSQLPostgres({
                host: 'cluster.dsql.us-east-1.on.aws',
                username: 'admin'
            });

            expect(mockDsqlSigner).toHaveBeenCalledTimes(1);
            const signerConfig = mockDsqlSigner.mock.calls[0][0];
            expect(signerConfig.region).toBe('us-east-1');
        });

        test('should handle hostname with insufficient parts', () => {
            AuroraDSQLPostgres({
                host: 'localhost',
                username: 'admin',
                region: 'us-east-1'
            });

            expect(mockDsqlSigner).toHaveBeenCalledTimes(1);
            const signerConfig = mockDsqlSigner.mock.calls[0][0];
            expect(signerConfig.region).toBe('us-east-1');
        });
    });

    describe('ClusterID as host', () => {
        test('should handle cluster ID given as hostname in connection string', () => {
            AuroraDSQLPostgres('postgres://admin@clusterID/', {
                region: 'us-east-1'
            });

            expect(mockPostgres).toHaveBeenCalledTimes(1);
            const options = mockPostgres.mock.calls[0][1];
            expect(options.host).toBe('clusterID.dsql.us-east-1.on.aws')
        });

        test('should handle cluster ID given as hostname in options', () => {
            AuroraDSQLPostgres({
                host: 'clusterID',
                region: 'us-east-1'
            });

            expect(mockPostgres).toHaveBeenCalledTimes(1);
            const options = mockPostgres.mock.calls[0][0];
            expect(options.host).toBe('clusterID.dsql.us-east-1.on.aws')
        });
    });

    describe('token generation', () => {
        test('should use admin token for admin user', async () => {
            AuroraDSQLPostgres({
                host: 'cluster.dsql.us-east-1.on.aws',
                username: 'admin',
                region: 'us-east-1'
            });

            const options = mockPostgres.mock.calls[0][0];
            const token = await options.pass();

            expect(token).toBe('admin-token');
        });

        test('should use regular token for non-admin user', async () => {
            AuroraDSQLPostgres({
                host: 'cluster.dsql.us-east-1.on.aws',
                username: 'testuser',
                region: 'us-east-1'
            });

            const options = mockPostgres.mock.calls[0][0];
            const token = await options.pass();

            expect(token).toBe('user-token');
        });
    });

    describe('function overloads', () => {
        test('should handle connection string + options', () => {
            const url = 'postgres://admin@cluster.dsql.us-east-1.on.aws/postgres';
            const options = {region: 'us-east-1', max: 5};

            AuroraDSQLPostgres(url, options);

            expect(mockPostgres).toHaveBeenCalledWith(url, expect.objectContaining({max: 5}));
        });

        test('should handle options only', () => {
            const options = {
                host: 'cluster.dsql.us-east-1.on.aws',
                username: 'admin',
                region: 'us-east-1',
                max: 10
            };

            AuroraDSQLPostgres(options);

            expect(mockPostgres).toHaveBeenCalledWith(expect.objectContaining({
                max: 10,
                host: 'cluster.dsql.us-east-1.on.aws'
            }));
        });
    });

    describe('DsqlSigner configuration', () => {
        test('should pass custom credentials provider', () => {
            const mockCredentialsProvider = {provide: async () => ({})};

            AuroraDSQLPostgres({
                host: 'cluster.dsql.us-east-1.on.aws',
                username: 'admin',
                region: 'us-east-1',
                customCredentialsProvider: mockCredentialsProvider
            });

            const signerConfig = mockDsqlSigner.mock.calls[0][0];
            expect(signerConfig.credentials).toBe(mockCredentialsProvider);
        });

        test('should pass expiresIn option', () => {
            AuroraDSQLPostgres({
                host: 'cluster.dsql.us-east-1.on.aws',
                username: 'admin',
                region: 'us-east-1',
                tokenDurationSecs: 3600
            });

            const signerConfig = mockDsqlSigner.mock.calls[0][0];
            expect(signerConfig.expiresIn).toBe(3600);
        });

        test('should accept profile option', () => {
            AuroraDSQLPostgres({
                host: 'cluster.dsql.us-east-1.on.aws',
                username: 'admin',
                region: 'us-east-1',
                profile: 'my-aws-profile'
            });

            expect(mockPostgres).toHaveBeenCalledTimes(1);
            expect(mockDsqlSigner).toHaveBeenCalledTimes(1);
            const signerConfig = mockDsqlSigner.mock.calls[0][0];
            expect(signerConfig.profile).toBe('my-aws-profile');
        });
    });
});
