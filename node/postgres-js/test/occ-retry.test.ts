/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import { jest, describe, test, beforeAll, beforeEach, expect } from '@jest/globals';
import { auroraDSQLPostgres } from "../src";

jest.mock('postgres', () => {
    const mockPostgres = jest.fn(() => {
        const sql = { end: jest.fn() } as any;
        sql.begin = jest.fn();
        return sql;
    });
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

function createOCCError(code: string, message = 'OCC conflict') {
    const error = new Error(message) as Error & { code: string; name: string };
    error.code = code;
    error.name = 'PostgresError';
    return error;
}

describe('OCC Retry via sql.transaction()', () => {
    let mockPostgres: any;
    let mockBegin: any;

    beforeAll(async () => {
        const postgresModule = await import('postgres');
        mockPostgres = postgresModule.default;
    });

    beforeEach(() => {
        jest.clearAllMocks();
    });

    function createSql(retryConfig?: any, logger?: any) {
        const sql = auroraDSQLPostgres({
            host: 'cluster.dsql.us-east-1.on.aws',
            username: 'admin',
            region: 'us-east-1',
            retry: retryConfig,
            logger: logger,
        });
        mockBegin = mockPostgres.mock.results[0].value.begin;
        return sql;
    }

    describe('successful transactions', () => {
        test('should execute transaction without retry on success', async () => {
            const sql = createSql();
            const callback = jest.fn().mockReturnValue('result');
            mockBegin.mockImplementation((fn: any) => fn());

            await sql.transaction(callback);

            expect(mockBegin).toHaveBeenCalledTimes(1);
            expect(callback).toHaveBeenCalledTimes(1);
        });

        test('should return value from transaction callback', async () => {
            const sql = createSql();
            mockBegin.mockResolvedValue('tx-result');

            const result = await sql.transaction(async () => 'tx-result');

            expect(result).toBe('tx-result');
        });
    });

    describe('OCC retry behavior', () => {
        test('should retry on OC000 data conflict', async () => {
            const sql = createSql({ maxRetries: 3 });
            const occError = createOCCError('OC000');
            mockBegin
                .mockRejectedValueOnce(occError)
                .mockResolvedValue('success');

            const result = await sql.transaction(async () => 'success');

            expect(result).toBe('success');
            expect(mockBegin).toHaveBeenCalledTimes(2);
        });

        test('should retry on OC001 schema conflict', async () => {
            const sql = createSql({ maxRetries: 3 });
            const occError = createOCCError('OC001');
            mockBegin
                .mockRejectedValueOnce(occError)
                .mockResolvedValue('success');

            const result = await sql.transaction(async () => 'success');

            expect(result).toBe('success');
            expect(mockBegin).toHaveBeenCalledTimes(2);
        });

        test('should retry on 40001 serialization failure', async () => {
            const sql = createSql({ maxRetries: 3 });
            const occError = createOCCError('40001', 'serialization failure');
            mockBegin
                .mockRejectedValueOnce(occError)
                .mockResolvedValue('success');

            const result = await sql.transaction(async () => 'success');

            expect(result).toBe('success');
            expect(mockBegin).toHaveBeenCalledTimes(2);
        });

        test('should not retry non-OCC errors', async () => {
            const sql = createSql({ maxRetries: 3 });
            const genericError = new Error('syntax error');
            mockBegin.mockRejectedValue(genericError);

            await expect(sql.transaction(async () => { })).rejects.toThrow('syntax error');
            expect(mockBegin).toHaveBeenCalledTimes(1);
        });

        test('should throw after max retries exhausted', async () => {
            const sql = createSql({ maxRetries: 3 });
            const occError = createOCCError('OC000');
            mockBegin.mockRejectedValue(occError);

            await expect(sql.transaction(async () => { })).rejects.toThrow('OCC conflict');
            // 3 retries + 1 initial = 4 total attempts
            expect(mockBegin).toHaveBeenCalledTimes(4);
        });

        test('should not retry when maxRetries is 0', async () => {
            const sql = createSql({ maxRetries: 0 });
            const occError = createOCCError('OC000');
            mockBegin.mockRejectedValue(occError);

            await expect(sql.transaction(async () => { })).rejects.toThrow('OCC conflict');
            expect(mockBegin).toHaveBeenCalledTimes(1);
        });
    });

    describe('per-call override', () => {
        test('should override constructor config with per-call maxRetries', async () => {
            const sql = createSql({ maxRetries: 3 });
            const occError = createOCCError('OC000');
            mockBegin.mockRejectedValue(occError);

            await expect(sql.transaction(async () => { }, { maxRetries: 5 })).rejects.toThrow('OCC conflict');
            // 5 retries + 1 initial = 6 total attempts
            expect(mockBegin).toHaveBeenCalledTimes(6);
        });

        test('should disable retry with per-call maxRetries 0', async () => {
            const sql = createSql({ maxRetries: 3 });
            const occError = createOCCError('OC000');
            mockBegin.mockRejectedValue(occError);

            await expect(sql.transaction(async () => { }, { maxRetries: 0 })).rejects.toThrow('OCC conflict');
            expect(mockBegin).toHaveBeenCalledTimes(1);
        });

        test('should allow per-call backoff override', async () => {
            const sql = createSql({ maxRetries: 2, baseDelayMs: 1, maxDelayMs: 10 });
            const occError = createOCCError('OC000');
            mockBegin.mockRejectedValue(occError);

            await expect(
                sql.transaction(async () => { }, { maxRetries: 1, baseDelayMs: 1, maxDelayMs: 50 })
            ).rejects.toThrow('OCC conflict');
            // per-call maxRetries = 1, so 2 total attempts
            expect(mockBegin).toHaveBeenCalledTimes(2);
        });
    });

    describe('begin options passthrough', () => {
        test('should pass begin options string to sql.begin()', async () => {
            const sql = createSql({ maxRetries: 3 });
            mockBegin.mockResolvedValue('result');

            await sql.transaction("read only", async () => 'result');

            expect(mockBegin).toHaveBeenCalledWith("read only", expect.any(Function));
        });

        test('should pass begin options with per-call retry override', async () => {
            const sql = createSql({ maxRetries: 3 });
            const occError = createOCCError('OC000');
            mockBegin
                .mockRejectedValueOnce(occError)
                .mockResolvedValue('success');

            const result = await sql.transaction("read only", async () => 'success', { maxRetries: 1 });

            expect(result).toBe('success');
            expect(mockBegin).toHaveBeenCalledTimes(2);
            expect(mockBegin).toHaveBeenCalledWith("read only", expect.any(Function));
        });
    });

    describe('default retry config', () => {
        test('should use default 3 retries when no retry config provided', async () => {
            const sql = createSql();
            const occError = createOCCError('OC000');
            mockBegin.mockRejectedValue(occError);

            await expect(sql.transaction(async () => { })).rejects.toThrow('OCC conflict');
            // default 3 retries + 1 initial = 4 total attempts
            expect(mockBegin).toHaveBeenCalledTimes(4);
        });
    });

    describe('logger integration', () => {
        test('should call logger.debug on each retry attempt', async () => {
            const logger = { debug: jest.fn(), warn: jest.fn(), error: jest.fn() };
            const sql = createSql({ maxRetries: 2 }, logger);
            const occError = createOCCError('OC000');
            mockBegin
                .mockRejectedValueOnce(occError)
                .mockResolvedValue('success');

            await sql.transaction(async () => 'success');

            expect(logger.debug).toHaveBeenCalledTimes(1);
            expect(logger.debug).toHaveBeenCalledWith(
                expect.stringContaining('OCC conflict on attempt 1/3')
            );
        });

        test('should call logger.error when retries exhausted', async () => {
            const logger = { debug: jest.fn(), warn: jest.fn(), error: jest.fn() };
            const sql = createSql({ maxRetries: 1 }, logger);
            const occError = createOCCError('OC000');
            mockBegin.mockRejectedValue(occError);

            await expect(sql.transaction(async () => { })).rejects.toThrow();

            expect(logger.error).toHaveBeenCalledTimes(1);
            expect(logger.error).toHaveBeenCalledWith(
                expect.stringContaining('OCC retry exhausted after 2 attempts')
            );
        });

        test('should work without logger (no crash)', async () => {
            const sql = createSql({ maxRetries: 1 });
            const occError = createOCCError('OC000');
            mockBegin.mockRejectedValue(occError);

            await expect(sql.transaction(async () => { })).rejects.toThrow();
        });
    });

    describe('retry config validation', () => {
        test('should reject negative maxRetries', () => {
            expect(() => createSql({ maxRetries: -1 })).toThrow('maxRetries must be >= 0');
        });

        test('should reject baseDelayMs <= 0', () => {
            expect(() => createSql({ baseDelayMs: 0 })).toThrow('baseDelayMs must be greater than 0');
        });

        test('should reject maxDelayMs < baseDelayMs', () => {
            expect(() => createSql({ baseDelayMs: 100, maxDelayMs: 10 })).toThrow('maxDelayMs must be >= baseDelayMs');
        });

        test('should reject jitterFactor < 0', () => {
            expect(() => createSql({ jitterFactor: -0.5 })).toThrow('jitterFactor must be between 0.0 and 1.0');
        });

        test('should reject jitterFactor > 1', () => {
            expect(() => createSql({ jitterFactor: 1.5 })).toThrow('jitterFactor must be between 0.0 and 1.0');
        });

        test('should reject maxRetries > 100', () => {
            expect(() => createSql({ maxRetries: 101 })).toThrow('maxRetries must not exceed 100');
        });

        test('should reject maxDelayMs > 100', () => {
            expect(() => createSql({ maxDelayMs: 101 })).toThrow('maxDelayMs must not exceed 100');
        });
    });
});
