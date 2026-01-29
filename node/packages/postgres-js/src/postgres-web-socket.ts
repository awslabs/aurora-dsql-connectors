/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

import { EventEmitter } from "events";
import { AuroraDSQLWsConfig } from "./client";
import { Mutex } from "async-mutex";

const HEARTBEAT_TIMEOUT = 5_000; // 5 seconds

const MESSAGE_CODE_SIMPLE_QUERY = "Q".charCodeAt(0);
const MESSAGE_CODE_SYNC = "S".charCodeAt(0);
const MESSAGE_CODE_RFQ = "Z".charCodeAt(0);

interface Query {
  buffer: Buffer | Uint8Array;
  numOfQueries: number;
  isHeartBeat: boolean;
}

enum ReadyState {
  Closed = 'closed',
  Querying = 'querying',
  Open = 'open'
}

export class PostgresWs extends EventEmitter {
  private mutex = new Mutex();
  private mutexRelease: (() => void) | null = null;

  private config: AuroraDSQLWsConfig<{}>;
  private ws: WebSocket | null = null;
  private connected: boolean = false;

  private disableHeartBeat: boolean = false;
  private heartBeatTimeout: NodeJS.Timeout | null = null;
  private pendingQueries: Query[] = [];

  public readyState: ReadyState = ReadyState.Closed;


  // these parameters are referenced when an Error is thrown
  private host: string;
  private port: number = 443;

  constructor(config: AuroraDSQLWsConfig<{}>) {
    super();
    this.config = config;
    this.host = config.host || '';
  }


  onReadyForQuery() {
    if (this.pendingQueries.length > 0) {
      if (this.pendingQueries[0].numOfQueries > 0) {
        this.pendingQueries[0].numOfQueries--;
      }

      // remove the pendingQueries now that we received the reply only if the querycount is at 0
      if (this.pendingQueries[0].numOfQueries <= 0) {
        this.pendingQueries.shift();
        this.readyState = ReadyState.Open;
        this.disableHeartBeat = false;
        this.releaseMutex();
        this.processQueue();
      }
    }
  }

  handleHeartBeatResponse(data: Uint8Array): boolean {
    if (this.pendingQueries.length === 0 || !this.pendingQueries[0].isHeartBeat) return false;

    const messageType = String.fromCharCode(data[0]);

    if (messageType === "D") {
      const dataStr = new TextDecoder().decode(data.slice(5));
      // verify that it is from heartbeat
      if (dataStr.includes("1")) {
        clearTimeout(this.heartBeatTimeout!);
        this.heartBeatTimeout = null;
      }
    } else if (messageType === "Z") {
      this.onReadyForQuery();
    }

    return true;
  }

  async connect(): Promise<this> {
    const url = `wss://${this.config.host}:${this.port}`;
    this.ws = new WebSocket(url);
    this.ws.binaryType = "arraybuffer";

    return new Promise((resolve, reject) => {
      if (this.ws != null) {
        this.ws.onopen = () => {
          this.connected = true;
          this.readyState = ReadyState.Open;
          resolve(this);
        };

        this.ws.onmessage = (event: MessageEvent) => {
          const data = new Uint8Array(event.data);

          if (this.config.connectionCheck) {
            if (this.handleHeartBeatResponse(data)) return;

            if (data.length > 0 && data[0] === MESSAGE_CODE_RFQ) {

              if (data.length > 5) {
                const status = String.fromCharCode(data[5]);

                if (status === "E") {
                  // temporarily disable heart beat when the connection has a transaction error
                  this.disableHeartBeat = true;
                  this.cleanUpTxErrorState();
                } else {
                  // I (IDLE) or T (Transaction) 
                  this.onReadyForQuery();
                }
              }
            }
          }

          this.emit("data", Buffer.from(data));
        };

        this.ws.onerror = (event: Event) => {
          const msg = (event as ErrorEvent).message || "WebSocket error";
          const error = new Error(`${msg} ${this.host}:${this.port}`);
          this.emit("error", error);
          reject(error);
        };

        this.ws.onclose = () => {
          if (this.heartBeatTimeout) {
            clearTimeout(this.heartBeatTimeout);
            this.heartBeatTimeout = null;
          }

          this.releaseMutex();

          this.connected = false;
          this.readyState = ReadyState.Closed;
          this.ws = null;
          this.emit("close");

          if (this.config.onReservedConnectionClose) {
            this.config.onReservedConnectionClose(this.config.connectionId);
          }
        };
      }
    });
  }

  createQueryBuffer(sql: string): Uint8Array {
    const sqlBytes = new TextEncoder().encode(sql + "\0");
    const length = 4 + sqlBytes.length;
    const buf = new Uint8Array(1 + length);
    const view = new DataView(buf.buffer);

    buf[0] = MESSAGE_CODE_SIMPLE_QUERY;
    view.setInt32(1, length, false);
    buf.set(sqlBytes, 5);

    return buf;
  }

  // future proofing in case postgres.js starts using the write function with 3 parameters
  // currently the library uses the write function with 2 parameters
  // callback is needed for copy related operations 
  write(data: Buffer | Uint8Array, callback?: (err?: Error) => void): boolean;
  write(data: string, encoding?: string, callback?: (err?: Error) => void): boolean;

  write(
    data: Buffer | Uint8Array | string,
    encodingOrCallback?: string | ((err?: Error) => void),
    callback?: (err?: Error) => void
  ): boolean {

    let callbackFn: ((err?: Error) => void) | undefined;

    if (typeof encodingOrCallback === 'function') {
      callbackFn = encodingOrCallback;
    } else {
      callbackFn = callback;
    }

    let sendData: Uint8Array;
    if (typeof data === 'string') {
      const encoder = new TextEncoder();
      sendData = encoder.encode(data);  // Always UTF-8
    } else if (data instanceof Buffer) {
      sendData = new Uint8Array(data);
    } else {
      sendData = data;
    }

    if (!this.ws || !this.connected) {
      if (callbackFn) {
        callbackFn(new Error("websocket is not connected"));
      }
      return false;
    }

    let offset = 0;

    let queryCount = 0; // simple query 
    let hasSync = false; // extended query protocol 

    while (offset < sendData.length) {
      if (sendData[offset] === MESSAGE_CODE_SIMPLE_QUERY) {
        queryCount++;
      }

      if (sendData[offset] === MESSAGE_CODE_SYNC) {
        hasSync = true;
      }

      // Read message length to skip to next message
      if (offset + 5 <= sendData.length) {
        const length = new DataView(sendData.buffer, sendData.byteOffset + offset + 1, 4).getInt32(0, false);
        offset += 1 + length;
      } else {
        break;
      }
    }

    // directly send message to websocket when 
    // * connection check (heart beat) is disabled 
    // * Z (ReadyForQuery) message is not expected to be returned
    if (!this.config.connectionCheck || (queryCount == 0 && hasSync === false)) {
      try {
        this.ws.send(sendData);
        if (callbackFn) {
          callbackFn();
        }
        return true;
      } catch (err) {
        if (callbackFn) {
          callbackFn(err as Error);
        }
        return false;
      }
    }

    // if the buffer contains a query, send a heart beat first
    if (
      queryCount > 0 &&
      this.config.connectionCheck &&
      this.heartBeatTimeout === null &&  // make sure no existing heart beat time out is overwritten
      !this.disableHeartBeat
    ) {
      const buf = this.createQueryBuffer("select 1;");
      this.pendingQueries.push({ buffer: buf, numOfQueries: 1, isHeartBeat: true });
    }
    this.pendingQueries.push({ buffer: sendData, numOfQueries: queryCount, isHeartBeat: false });
    this.processQueue();

    if (callbackFn) {
      callbackFn();
    }

    return true;
  }

  private releaseMutex() {
    if (this.mutexRelease) {
      this.mutexRelease();
      this.mutexRelease = null;
    }
  }

  // cleans up the error state during a transaction error 
  private cleanUpTxErrorState() {
    // throw away last query as the result set has a transaction error and no more result set should return 
    if (this.pendingQueries.length > 0) {
      this.pendingQueries.shift();
    }

    // since heart beat cant be performed during a transaction error, discard heart beat queries which are already in pending queries
    while (this.pendingQueries.length > 0 && this.pendingQueries[0].isHeartBeat) {
      this.pendingQueries.shift();
    }
    this.readyState = ReadyState.Open;
    this.releaseMutex();

    if (this.pendingQueries.length > 0) {
      this.processQueue();
    }
  }

  private async processQueue(): Promise<void> {
    if (
      this.mutexRelease ||
      this.readyState !== ReadyState.Open ||
      this.pendingQueries.length === 0
    ) {
      return;
    }

    this.mutexRelease = await this.mutex.acquire();

    // it is possible that the pendingQueries is now empty 
    if (this.pendingQueries.length == 0) {
      this.releaseMutex();
      return;
    }

    const data = this.pendingQueries[0];

    if (data.buffer.length > 0 && data.buffer[0] === MESSAGE_CODE_SIMPLE_QUERY) {
      this.readyState = ReadyState.Querying;
    }

    if (data.isHeartBeat) {
      this.heartBeatTimeout = setTimeout(() => {
        console.log("Heart beat timed out");
        if (this.ws) {
          this.ws.close(1000, "Heart beat timeout");
        }
      }, HEARTBEAT_TIMEOUT);
    }

    if (this.ws) {
      this.ws.send(data.buffer);
    } else {
      throw Error("Websocket is not initialized");
    }
  }

  end(): void {
    if (this.ws && this.readyState !== ReadyState.Closed) {
      this.ws.close();
    }
  }

  destroy(): void {
    if (this.ws) {
      this.ws.close();
    }
  }

  // Following functions needs to be defined as connection.js 
  // in postgres.js can call these functions but they will be no-op
  setKeepAlive(): this {
    return this;
  }

  resume(): this {
    return this;
  }

  pause(): this {
    return this;
  }

  cork(): this {
    return this;
  }

  uncork(): this {
    return this;
  }
}

export function createPostgresWs(
  config: AuroraDSQLWsConfig<{}>
): () => Promise<PostgresWs> {
  return async () => {

    const socket = new PostgresWs(config);
    await socket.connect();
    return socket;
  };
}
