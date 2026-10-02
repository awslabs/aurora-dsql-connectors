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
  readonly sent: ArrayBuffer[] = [];

  constructor(url: string) {
    this.url = url;
    MockWebSocket.instances.push(this);
  }

  send(data: ArrayBuffer): void {
    this.sent.push(data);
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

function pgMessage(type: string, body: string = ''): Uint8Array {
  const bytes = new TextEncoder().encode(body);
  const buf = new Uint8Array(5 + bytes.length);
  buf[0] = type.charCodeAt(0);
  new DataView(buf.buffer).setInt32(1, 4 + bytes.length, false);
  buf.set(bytes, 5);
  return buf;
}

function concat(...parts: Uint8Array[]): Uint8Array {
  return new Uint8Array(Buffer.concat(parts));
}

// A complete simple-query reply: RowDescription, DataRow, CommandComplete, ReadyForQuery.
function queryReply(value: string): Uint8Array {
  return concat(pgMessage('T', 'x'), pgMessage('D', value), pgMessage('C', 'SELECT 1\0'), pgMessage('Z', 'I'));
}

const flush = () => new Promise((resolve) => setImmediate(resolve));

describe('PostgresWs with connectionCheck', () => {
  let socket: PostgresWs;
  let ws: MockWebSocket;

  beforeEach(async () => {
    MockWebSocket.instances = [];
    globalThis.WebSocket = MockWebSocket as unknown as typeof WebSocket;
    const config = { host: HOST, connectionCheck: true } as AuroraDSQLWsConfig<Record<string, never>>;
    socket = new PostgresWs(config as AuroraDSQLWsConfig<{}>);
    const connecting = socket.connect();
    ws = MockWebSocket.instances[0];
    ws.onopen!();
    await connecting;
  });

  afterEach(() => {
    ws.onclose?.();
    globalThis.WebSocket = originalWebSocket;
  });

  const deliver = (data: Uint8Array) =>
    ws.onmessage!({ data: data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength) } as MessageEvent);

  test('sends the query once a heartbeat reply arrives as one frame', async () => {
    const query = socket.createQueryBuffer('select 2;');
    socket.write(query);
    await flush();
    expect(ws.sent).toEqual([socket.createQueryBuffer('select 1;').buffer]);

    deliver(queryReply('1'));
    await flush();

    expect(ws.sent).toEqual([socket.createQueryBuffer('select 1;').buffer, query.buffer]);
  });

  test('forwards a reply that arrives as one frame and accepts the next query', async () => {
    const heartbeat = socket.createQueryBuffer('select 1;');
    const query = socket.createQueryBuffer('select 2;');
    socket.write(query);
    await flush();
    deliver(queryReply('1'));
    await flush();
    const received: Buffer[] = [];
    socket.on('data', (chunk: Buffer) => received.push(chunk));

    deliver(queryReply('2'));
    socket.write(socket.createQueryBuffer('select 3;'));
    await flush();

    expect(Buffer.concat(received)).toEqual(Buffer.from(queryReply('2')));
    // The next query starts with its own heartbeat.
    expect(ws.sent).toEqual([heartbeat.buffer, query.buffer, heartbeat.buffer]);
  });

  test('does not forward a heartbeat reply to postgres.js', async () => {
    socket.write(socket.createQueryBuffer('select 2;'));
    await flush();
    const received: Buffer[] = [];
    socket.on('data', (chunk: Buffer) => received.push(chunk));

    deliver(queryReply('1'));
    await flush();

    expect(received).toEqual([]);
  });

  test('reassembles a heartbeat reply split across frames', async () => {
    const query = socket.createQueryBuffer('select 2;');
    socket.write(query);
    await flush();
    const reply = queryReply('1');

    deliver(reply.subarray(0, 9));
    deliver(reply.subarray(9));
    await flush();

    expect(ws.sent).toEqual([socket.createQueryBuffer('select 1;').buffer, query.buffer]);
  });
});
