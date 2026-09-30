/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
import { describe, test, beforeEach, afterEach, expect } from '@jest/globals';
import { PostgresWs } from '../src/postgres-web-socket';
import { AuroraDSQLWsConfig } from '../src/client';

const HOST = 'cluster.dsql.us-east-1.on.aws';

class MockWebSocket {
  static instances: MockWebSocket[] = [];

  binaryType = '';
  onopen: (() => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;
  onclose: (() => void) | null = null;

  readonly url: string;

  constructor(url: string) {
    this.url = url;
    MockWebSocket.instances.push(this);
  }

  send(): void {
    // no-op
  }

  close(): void {
    // no-op
  }
}

const originalWebSocket = globalThis.WebSocket;

function newSocket(): { socket: PostgresWs; connecting: Promise<PostgresWs>; ws: MockWebSocket } {
  const config = { host: HOST } as AuroraDSQLWsConfig<Record<string, never>>;
  const socket = new PostgresWs(config as AuroraDSQLWsConfig<{}>);
  // connect() has no awaits before it assigns the handlers, so they are attached
  // synchronously and can be driven directly once it returns.
  const connecting = socket.connect();
  const ws = MockWebSocket.instances[MockWebSocket.instances.length - 1];
  return { socket, connecting, ws };
}

describe('PostgresWs handshake failures', () => {
  beforeEach(() => {
    MockWebSocket.instances = [];
    globalThis.WebSocket = MockWebSocket as unknown as typeof WebSocket;
  });

  afterEach(() => {
    globalThis.WebSocket = originalWebSocket;
  });

  test('rejects connect() when the handshake errors', async () => {
    const { connecting, ws } = newSocket();

    ws.onerror!({ message: 'Connection refused' } as unknown as Event);

    await expect(connecting).rejects.toThrow(`Connection refused ${HOST}:443`);
  });

  test('does not throw on the WebSocket callback stack when the handshake errors', () => {
    const { connecting, ws } = newSocket();
    // Swallow the expected rejection so it does not surface as an unhandled rejection.
    connecting.catch(() => undefined);

    // Regression: emitting "error" here while postgres.js has not yet attached its listener
    // made EventEmitter rethrow synchronously, escaping to window.onerror / uncaughtException.
    expect(() => ws.onerror!({ message: 'Connection refused' } as unknown as Event)).not.toThrow();
  });

  test('emits "error" for failures after the connection is established', async () => {
    const { socket, connecting, ws } = newSocket();

    ws.onopen!();
    await expect(connecting).resolves.toBe(socket);

    const received: Error[] = [];
    socket.on('error', (err: Error) => received.push(err));

    ws.onerror!({ message: 'Socket hang up' } as unknown as Event);

    expect(received).toHaveLength(1);
    expect(received[0].message).toBe(`Socket hang up ${HOST}:443`);
  });

  test('falls back to a default message when the error event carries none', async () => {
    const { connecting, ws } = newSocket();

    ws.onerror!({} as unknown as Event);

    await expect(connecting).rejects.toThrow(`WebSocket error ${HOST}:443`);
  });
});
