// ============================================================
// wasi:io shim - stream types for the wasi runtime
// ============================================================
//
// wasi uses streams extensively for i/o operations.
// the filesystem uses streams to read/write file contents.
// sockets use streams for network data transfer.
//
// this file provides minimal implementations of:
// - wasi:io/streams (input-stream, output-stream)
// - wasi:io/error (error type)
//
// in a full implementation, these would integrate with node.js
// streams. for our mock, we keep it simple.
// ============================================================

// ------------------------------------------------------------
// wasi:io/error implementation
// ------------------------------------------------------------
// defines the error type used by stream operations

export class IoError {
    #message;

    constructor(message) {
        this.#message = message;
    }

    toDebugString() {
        return this.#message;
    }
}

// ------------------------------------------------------------
// wasi:io/streams implementation
// ------------------------------------------------------------
// input-stream and output-stream resources

export class InputStream {
    #data;
    #position;

    constructor(data = new Uint8Array(0)) {
        this.#data = data;
        this.#position = 0;
    }

    // read: read up to len bytes from the stream
    read(len) {
        const remaining = this.#data.length - this.#position;
        const toRead = Math.min(Number(len), remaining);

        if (toRead === 0) {
            // end of stream
            return { tag: 'ok', val: new Uint8Array(0) };
        }

        const chunk = this.#data.slice(this.#position, this.#position + toRead);
        this.#position += toRead;

        return { tag: 'ok', val: chunk };
    }

    // blocking-read: same as read but blocks until data available
    blockingRead(len) {
        return this.read(len);
    }

    // skip: skip over bytes without reading them
    skip(len) {
        const remaining = this.#data.length - this.#position;
        const toSkip = Math.min(Number(len), remaining);
        this.#position += toSkip;
        return { tag: 'ok', val: BigInt(toSkip) };
    }

    // subscribe: get a pollable for this stream
    subscribe() {
        // simplified - return a mock pollable
        return new Pollable();
    }
}

export class OutputStream {
    #chunks;

    constructor() {
        this.#chunks = [];
    }

    // write: write bytes to the stream
    write(data) {
        this.#chunks.push(data);
        return { tag: 'ok', val: BigInt(data.length) };
    }

    // blocking-write-and-flush: write and ensure data is sent
    blockingWriteAndFlush(data) {
        return this.write(data);
    }

    // flush: ensure all buffered data is sent
    flush() {
        return { tag: 'ok', val: undefined };
    }

    // blocking-flush: blocking version of flush
    blockingFlush() {
        return this.flush();
    }

    // check-write: how many bytes can be written without blocking
    checkWrite() {
        // we can always write (no buffering limit)
        return { tag: 'ok', val: BigInt(1024 * 1024) };
    }

    // subscribe: get a pollable for this stream
    subscribe() {
        return new Pollable();
    }

    // get the written data (for testing)
    getData() {
        return this.#chunks;
    }
}

// ------------------------------------------------------------
// pollable - simplified implementation
// ------------------------------------------------------------
// in real wasi, pollables are used for async i/o.
// we provide a minimal mock here.

export class Pollable {
    ready() {
        return true;
    }

    block() {
        // no-op in our sync implementation
    }
}

// convenience function to create streams
export function createInputStream(data) {
    return new InputStream(data);
}

export function createOutputStream() {
    return new OutputStream();
}
