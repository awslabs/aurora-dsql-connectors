/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import { describe, test, expect } from '@jest/globals';
import { resolveRetryConfig, isOCCError } from '../src/occ-retry';

describe('OCC Retry', () => {
  describe('isOCCError', () => {
    test('should detect OCC error codes', () => {
      expect(isOCCError(Object.assign(new Error(), { code: 'OC000' }))).toBe(true);
      expect(isOCCError(Object.assign(new Error(), { code: 'OC001' }))).toBe(true);
      expect(isOCCError(Object.assign(new Error(), { code: '40001' }))).toBe(true);
    });

    test('should reject non-OCC errors', () => {
      expect(isOCCError(Object.assign(new Error(), { code: '42601' }))).toBe(false);
      expect(isOCCError(new Error('generic'))).toBe(false);
      expect(isOCCError(null)).toBe(false);
    });
  });

  describe('resolveRetryConfig', () => {
    test('should return null when not opted in', () => {
      expect(resolveRetryConfig(undefined, undefined)).toBeNull();
      expect(resolveRetryConfig(false, undefined)).toBeNull();
      expect(resolveRetryConfig({ maxRetries: 5 }, false)).toBeNull();
    });

    test('should return defaults when opted in with true', () => {
      expect(resolveRetryConfig(true, undefined)).toEqual({
        maxRetries: 3, baseDelayMs: 1, maxDelayMs: 100, jitterFactor: 0.25,
      });
    });

    test('should merge partial config with defaults', () => {
      const config = resolveRetryConfig({ maxRetries: 5 }, undefined);
      expect(config!.maxRetries).toBe(5);
      expect(config!.baseDelayMs).toBe(1);
    });

    test('should override constructor config with per-call config', () => {
      expect(resolveRetryConfig({ maxRetries: 3 }, { maxRetries: 7 })!.maxRetries).toBe(7);
    });
  });

  describe('retry config validation', () => {
    test('should reject invalid configs', () => {
      expect(() => resolveRetryConfig({ maxRetries: -1 }, undefined)).toThrow('maxRetries must be >= 0');
      expect(() => resolveRetryConfig({ maxRetries: 101 }, undefined)).toThrow('maxRetries must not exceed 100');
      expect(() => resolveRetryConfig({ baseDelayMs: 0 }, undefined)).toThrow('baseDelayMs must be greater than 0');
      expect(() => resolveRetryConfig({ baseDelayMs: 50, maxDelayMs: 10 }, undefined)).toThrow('maxDelayMs must be >= baseDelayMs');
      expect(() => resolveRetryConfig({ jitterFactor: -0.5 }, undefined)).toThrow('jitterFactor must be between 0.0 and 1.0');
      expect(() => resolveRetryConfig({ jitterFactor: 1.5 }, undefined)).toThrow('jitterFactor must be between 0.0 and 1.0');
    });
  });
});
