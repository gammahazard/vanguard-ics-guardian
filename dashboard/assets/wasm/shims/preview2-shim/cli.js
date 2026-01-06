// ============================================================
// cli.js - minimal browser-compatible wasi:cli stubs
// ============================================================
//
// provides minimal implementations of wasi:cli interfaces for browser.
// these stubs satisfy jco's wasi imports for our guest component.
// the guest primarily uses our custom interfaces (sensor-*),
// so standard wasi interfaces are minimally implemented.
//
// related files:
// - filesystem.js: wasi:filesystem stubs
// - io.js: wasi:io stubs (streams, error)
// - malicious_driver.js: imports from this module
// ============================================================

// wasi:cli/environment stub
export const environment = {
    getEnvironment: () => []
};

// wasi:cli/exit stub
export const exit = {
    exit: (code) => {
        console.log('[wasi] exit called with code:', code);
    }
};

// wasi:cli/stderr stub
export const stderr = {
    getStderr: () => ({
        write: (data) => {
            console.error(new TextDecoder().decode(data));
            return { tag: 'ok', val: BigInt(data.length) };
        },
        blockingFlush: () => ({ tag: 'ok', val: undefined })
    })
};

// wasi:cli/stdin stub
export const stdin = {
    getStdin: () => ({
        read: () => ({ tag: 'ok', val: new Uint8Array(0) })
    })
};

// wasi:cli/stdout stub
export const stdout = {
    getStdout: () => ({
        write: (data) => {
            console.log(new TextDecoder().decode(data));
            return { tag: 'ok', val: BigInt(data.length) };
        },
        blockingFlush: () => ({ tag: 'ok', val: undefined })
    })
};
