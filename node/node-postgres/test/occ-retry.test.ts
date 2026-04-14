/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import {
  OCCType,
  isOccError,
  calculateBackoff,
  executeWithRetry,
  OccRetryExhaustedError,
  DEFAULT_OCC_CONFIG,
  OccRetryConfig,
  OccRetryEvent,
  OccRetryExhaustedEvent,
  validateOccConfig
} from '../src/occ-retry';

// Mock database error helper
function createDbError(code: string, message: string): Error {
  const error = new Error(message) as any;
  error.code = code;
  return error;
}

describe('OCC Retry - Core Feature Tests', () => {
  describe('isOccError - OCC Error Detection', () => {
    it('should detect OC000 data conflict', () => {
      const error = createDbError('OC000', 'concurrent data modification');
      const result = isOccError(error);

      expect(result).toEqual({ type: OCCType.Data, code: 'OC000' });
    });

    it('should detect OC001 schema conflict', () => {
      const error = createDbError('OC001', 'DDL during transaction');
      const result = isOccError(error);

      expect(result).toEqual({ type: OCCType.Schema, code: 'OC001' });
    });

    it('should detect 40001 with embedded (OC000) in message', () => {
      const error = createDbError('40001', 'serialization failure (OC000)');
      const result = isOccError(error);

      expect(result).toEqual({ type: OCCType.Data, code: 'OC000' });
    });

    it('should detect 40001 with embedded (OC001) in message', () => {
      const error = createDbError('40001', 'serialization failure (OC001)');
      const result = isOccError(error);

      expect(result).toEqual({ type: OCCType.Schema, code: 'OC001' });
    });

    it('should detect 40001 without embedded code as Unknown', () => {
      const error = createDbError('40001', 'serialization failure');
      const result = isOccError(error);

      expect(result).toEqual({ type: OCCType.Unknown, code: '40001' });
    });

    it('should return null for non-OCC errors', () => {
      const error = createDbError('23505', 'unique violation');
      expect(isOccError(error)).toBeNull();
    });

    it('should return null for errors without code', () => {
      const error = new Error('generic error');
      expect(isOccError(error)).toBeNull();
    });
  });

  describe('calculateBackoff - Exponential Backoff', () => {
    const config: Required<OccRetryConfig> = {
      enabled: true,
      maxAttempts: 3,
      baseDelayMs: 10,
      maxDelayMs: 100,
      jitterFactor: 0.25
    };

    it('should calculate exponential backoff correctly', () => {
      // Attempt 1: 10 * 2^0 = 10ms, jitter up to 2.5ms
      const delay1 = calculateBackoff(config, 1);
      expect(delay1).toBeGreaterThanOrEqual(10);
      expect(delay1).toBeLessThanOrEqual(13);

      // Attempt 2: 10 * 2^1 = 20ms, jitter up to 5ms
      const delay2 = calculateBackoff(config, 2);
      expect(delay2).toBeGreaterThanOrEqual(20);
      expect(delay2).toBeLessThanOrEqual(26);

      // Attempt 3: 10 * 2^2 = 40ms, jitter up to 10ms
      const delay3 = calculateBackoff(config, 3);
      expect(delay3).toBeGreaterThanOrEqual(40);
      expect(delay3).toBeLessThanOrEqual(51);
    });

    it('should respect maxDelayMs cap with jitter', () => {
      const delay = calculateBackoff(config, 10);
      // 10 * 2^9 = 5120ms, capped at 100ms, then jitter added (up to 25ms)
      expect(delay).toBeGreaterThanOrEqual(100);
      expect(delay).toBeLessThanOrEqual(125);
    });

    it('should work with zero jitter', () => {
      const noJitterConfig = { ...config, jitterFactor: 0 };

      expect(calculateBackoff(noJitterConfig, 1)).toBe(10);
      expect(calculateBackoff(noJitterConfig, 2)).toBe(20);
      expect(calculateBackoff(noJitterConfig, 3)).toBe(40);
    });
  });

  describe('executeWithRetry - Core Retry Logic', () => {
    const config: Required<OccRetryConfig> = {
      enabled: true,
      maxAttempts: 3,
      baseDelayMs: 1,
      maxDelayMs: 10,
      jitterFactor: 0
    };

    let events: Array<OccRetryEvent | OccRetryExhaustedEvent>;

    beforeEach(() => {
      events = [];
    });

    const captureEvent = (event: OccRetryEvent | OccRetryExhaustedEvent) => {
      events.push(event);
    };

    it('should succeed on first attempt without retry', async () => {
      const operation = jest.fn().mockResolvedValue('success');

      const result = await executeWithRetry(operation, config, captureEvent);

      expect(result).toBe('success');
      expect(operation).toHaveBeenCalledTimes(1);
      expect(events).toHaveLength(0);
    });

    it('should retry on OCC error and eventually succeed', async () => {
      const operation = jest.fn()
        .mockRejectedValueOnce(createDbError('OC000', 'conflict'))
        .mockResolvedValueOnce('success');

      const result = await executeWithRetry(operation, config, captureEvent);

      expect(result).toBe('success');
      expect(operation).toHaveBeenCalledTimes(2);
      expect(events).toHaveLength(1);
      expect(events[0].type).toBe('occRetry');
    });

    it('should retry multiple times on repeated OCC errors', async () => {
      const operation = jest.fn()
        .mockRejectedValueOnce(createDbError('OC000', 'conflict 1'))
        .mockRejectedValueOnce(createDbError('OC000', 'conflict 2'))
        .mockResolvedValueOnce('success');

      const result = await executeWithRetry(operation, config, captureEvent);

      expect(result).toBe('success');
      expect(operation).toHaveBeenCalledTimes(3);
      expect(events).toHaveLength(2);
      expect(events[0].type).toBe('occRetry');
      expect(events[1].type).toBe('occRetry');
    });

    it('should throw non-OCC errors immediately without retry', async () => {
      const nonOccError = createDbError('23505', 'unique violation');
      const operation = jest.fn().mockRejectedValue(nonOccError);

      await expect(
        executeWithRetry(operation, config, captureEvent)
      ).rejects.toThrow('unique violation');

      expect(operation).toHaveBeenCalledTimes(1);
      expect(events).toHaveLength(0);
    });

    it('should throw OccRetryExhaustedError after max attempts', async () => {
      const occError = createDbError('OC000', 'persistent conflict');
      const operation = jest.fn().mockRejectedValue(occError);

      await expect(
        executeWithRetry(operation, config, captureEvent)
      ).rejects.toThrow(OccRetryExhaustedError);

      expect(operation).toHaveBeenCalledTimes(3);
      expect(events).toHaveLength(3); // 2 retry events + 1 exhausted event

      const exhaustedEvent = events[2] as OccRetryExhaustedEvent;
      expect(exhaustedEvent.type).toBe('occRetryExhausted');
      expect(exhaustedEvent.attempts).toBe(3);
    });

    it('should emit occRetry events with correct metadata', async () => {
      const operation = jest.fn()
        .mockRejectedValueOnce(createDbError('OC001', 'schema conflict'))
        .mockResolvedValueOnce('success');

      await executeWithRetry(operation, config, captureEvent, 'UPDATE test');

      const retryEvent = events[0] as OccRetryEvent;
      expect(retryEvent.type).toBe('occRetry');
      expect(retryEvent.attempt).toBe(2);
      expect(retryEvent.maxAttempts).toBe(3);
      expect(retryEvent.occType).toBe(OCCType.Schema);
      expect(retryEvent.occCode).toBe('OC001');
      expect(retryEvent.queryText).toBe('UPDATE test');
      expect(retryEvent.delayMs).toBeGreaterThanOrEqual(0);
    });

    it('should emit occRetryExhausted event with correct metadata', async () => {
      const occError = createDbError('40001', 'serialization failure');
      const operation = jest.fn().mockRejectedValue(occError);

      await expect(
        executeWithRetry(operation, config, captureEvent, 'SELECT * FROM test')
      ).rejects.toThrow(OccRetryExhaustedError);

      const exhaustedEvent = events[2] as OccRetryExhaustedEvent;
      expect(exhaustedEvent.type).toBe('occRetryExhausted');
      expect(exhaustedEvent.attempts).toBe(3);
      expect(exhaustedEvent.occType).toBe(OCCType.Unknown);
      expect(exhaustedEvent.occCode).toBe('40001');
      expect(exhaustedEvent.queryText).toBe('SELECT * FROM test');
    });

    it('should handle maxAttempts of 1 (no retry)', async () => {
      const singleConfig = { ...config, maxAttempts: 1 };
      const occError = createDbError('OC000', 'conflict');
      const operation = jest.fn().mockRejectedValue(occError);

      await expect(
        executeWithRetry(operation, singleConfig, captureEvent)
      ).rejects.toThrow(OccRetryExhaustedError);

      expect(operation).toHaveBeenCalledTimes(1);
      expect(events).toHaveLength(1);
      expect(events[0].type).toBe('occRetryExhausted');
    });
  });

  describe('OccRetryExhaustedError', () => {
    it('should construct with correct properties', () => {
      const lastError = createDbError('OC000', 'conflict');
      const occInfo = { type: OCCType.Data, code: 'OC000' };

      const error = new OccRetryExhaustedError(3, lastError, occInfo);

      expect(error.name).toBe('OccRetryExhaustedError');
      expect(error.attempts).toBe(3);
      expect(error.lastError).toBe(lastError);
      expect(error.occInfo).toEqual(occInfo);
      expect(error.message).toContain('3 attempts');
      expect(error.message).toContain('Data');
      expect(error.message).toContain('OC000');
    });

    it('should be instanceof OccRetryExhaustedError', () => {
      const error = new OccRetryExhaustedError(
        3,
        new Error('test'),
        { type: OCCType.Data, code: 'OC000' }
      );

      expect(error instanceof OccRetryExhaustedError).toBe(true);
      expect(error instanceof Error).toBe(true);
    });
  });

  describe('DEFAULT_OCC_CONFIG', () => {
    it('should match Rust implementation defaults', () => {
      expect(DEFAULT_OCC_CONFIG).toEqual({
        enabled: false,
        maxAttempts: 3,
        baseDelayMs: 1,
        maxDelayMs: 100,
        jitterFactor: 0.25
      });
    });
  });

  describe('OCCType enum', () => {
    it('should have correct values', () => {
      expect(OCCType.Data).toBe('Data');
      expect(OCCType.Schema).toBe('Schema');
      expect(OCCType.Unknown).toBe('Unknown');
    });
  });

  describe('validateOccConfig', () => {
    it('should accept valid config', () => {
      expect(() => validateOccConfig({
        enabled: true,
        maxAttempts: 3,
        baseDelayMs: 1,
        maxDelayMs: 100,
        jitterFactor: 0.25
      })).not.toThrow();
    });

    it('should reject maxAttempts less than 1', () => {
      expect(() => validateOccConfig({
        enabled: true,
        maxAttempts: 0,
        baseDelayMs: 1,
        maxDelayMs: 100,
        jitterFactor: 0.25
      })).toThrow('occ.maxAttempts must be between 1 and 100');
    });

    it('should reject maxAttempts greater than 100', () => {
      expect(() => validateOccConfig({
        enabled: true,
        maxAttempts: 101,
        baseDelayMs: 1,
        maxDelayMs: 100,
        jitterFactor: 0.25
      })).toThrow('occ.maxAttempts must be between 1 and 100');
    });

    it('should reject baseDelayMs less than 1', () => {
      expect(() => validateOccConfig({
        enabled: true,
        maxAttempts: 3,
        baseDelayMs: 0,
        maxDelayMs: 100,
        jitterFactor: 0.25
      })).toThrow('occ.baseDelayMs must be at least 1');
    });

    it('should reject maxDelayMs greater than 100', () => {
      expect(() => validateOccConfig({
        enabled: true,
        maxAttempts: 3,
        baseDelayMs: 1,
        maxDelayMs: 101,
        jitterFactor: 0.25
      })).toThrow('occ.maxDelayMs must not exceed 100');
    });

    it('should reject maxDelayMs less than baseDelayMs', () => {
      expect(() => validateOccConfig({
        enabled: true,
        maxAttempts: 3,
        baseDelayMs: 50,
        maxDelayMs: 10,
        jitterFactor: 0.25
      })).toThrow('occ.maxDelayMs must be >= occ.baseDelayMs');
    });

    it('should reject jitterFactor less than 0', () => {
      expect(() => validateOccConfig({
        enabled: true,
        maxAttempts: 3,
        baseDelayMs: 1,
        maxDelayMs: 100,
        jitterFactor: -0.1
      })).toThrow('occ.jitterFactor must be between 0 and 1');
    });

    it('should reject jitterFactor greater than 1', () => {
      expect(() => validateOccConfig({
        enabled: true,
        maxAttempts: 3,
        baseDelayMs: 1,
        maxDelayMs: 100,
        jitterFactor: 1.5
      })).toThrow('occ.jitterFactor must be between 0 and 1');
    });
  });
});
