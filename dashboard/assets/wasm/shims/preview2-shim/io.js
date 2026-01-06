// ============================================================
// io.js - minimal browser-compatible wasi:io stubs
// ============================================================
//
// provides minimal implementations of wasi:io interfaces for browser.
// includes input/output stream classes and error types.
// used by jco-generated bindings for standard wasi io operations.
//
// related files:
// - cli.js: wasi:cli stubs (uses streams from here)
// - filesystem.js: wasi:filesystem stubs
// - malicious_driver.js: imports from this module
// ============================================================

export class IoError {
    constructor(message) {
        this.message = message;
    }

    toDebugString() {
        return this.message;
    }
}

export class InputStream {
    constructor(data = new Uint8Array(0)) {
        this.data = data;
        this.position = 0;
    }

    read(len) {
        const remaining = this.data.length - this.position;
        const toRead = Math.min(Number(len), remaining);
        if (toRead === 0) {
            return { tag: 'ok', val: new Uint8Array(0) };
        }
        const chunk = this.data.slice(this.position, this.position + toRead);
        this.position += toRead;
        return { tag: 'ok', val: chunk };
    }

    blockingRead(len) {
        return this.read(len);
    }

    skip(len) {
        const remaining = this.data.length - this.position;
        const toSkip = Math.min(Number(len), remaining);
        this.position += toSkip;
        return { tag: 'ok', val: BigInt(toSkip) };
    }

    subscribe() {
        return { ready: () => true, block: () => { } };
    }
}

export class OutputStream {
    constructor() {
        this.chunks = [];
    }

    write(data) {
        this.chunks.push(data);
        return { tag: 'ok', val: BigInt(data.length) };
    }

    blockingWriteAndFlush(data) {
        return this.write(data);
    }

    flush() {
        return { tag: 'ok', val: undefined };
    }

    blockingFlush() {
        return this.flush();
    }

    checkWrite() {
        return { tag: 'ok', val: BigInt(1024 * 1024) };
    }

    subscribe() {
        return { ready: () => true, block: () => { } };
    }
}

export const error = {
    Error: IoError
};

export const streams = {
    InputStream,
    OutputStream
};
