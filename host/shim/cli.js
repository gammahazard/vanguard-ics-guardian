// ============================================================
// wasi:cli shim - console output for the guest component
// ============================================================
//
// provides stdout and stderr streams so the guest can print
// messages. this is how the malicious driver reports its actions.
//
// we simply pipe these to node's console for visibility.
// ============================================================

import { OutputStream } from './io.js';

// ------------------------------------------------------------
// custom output stream that writes to console
// ------------------------------------------------------------

class ConsoleOutputStream extends OutputStream {
    #isStderr;
    #buffer;

    constructor(isStderr = false) {
        super();
        this.#isStderr = isStderr;
        this.#buffer = '';
    }

    // override write to output to console
    write(data) {
        // decode the bytes to a string
        const decoder = new TextDecoder();
        const text = decoder.decode(data);

        // buffer the text and flush on newlines
        this.#buffer += text;

        // output complete lines
        const lines = this.#buffer.split('\n');
        this.#buffer = lines.pop(); // keep incomplete line in buffer

        for (const line of lines) {
            if (this.#isStderr) {
                console.error(line);
            } else {
                console.log(line);
            }
        }

        return { tag: 'ok', val: BigInt(data.length) };
    }

    // flush any remaining buffered content
    blockingFlush() {
        if (this.#buffer) {
            if (this.#isStderr) {
                console.error(this.#buffer);
            } else {
                console.log(this.#buffer);
            }
            this.#buffer = '';
        }
        return { tag: 'ok', val: undefined };
    }
}

// singleton instances
const stdout = new ConsoleOutputStream(false);
const stderr = new ConsoleOutputStream(true);

// ------------------------------------------------------------
// wasi:cli/stdout implementation
// ------------------------------------------------------------

export function getStdout() {
    return stdout;
}

// ------------------------------------------------------------
// wasi:cli/stderr implementation
// ------------------------------------------------------------

export function getStderr() {
    return stderr;
}
