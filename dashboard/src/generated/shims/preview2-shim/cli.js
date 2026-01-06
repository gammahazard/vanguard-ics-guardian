// ============================================================
// Minimal browser-compatible preview2-shim stubs
// ============================================================
// These stubs satisfy jco's WASI imports for our guest component.
// The guest primarily uses our custom interfaces (sensor-*).
// Standard WASI interfaces are minimally implemented.

// wasi:cli stubs
export const environment = {
    getEnvironment: () => []
};

export const exit = {
    exit: (code) => {
        console.log('[WASI] exit called with code:', code);
    }
};

export const stderr = {
    getStderr: () => ({
        write: (data) => {
            console.error(new TextDecoder().decode(data));
            return { tag: 'ok', val: BigInt(data.length) };
        },
        blockingFlush: () => ({ tag: 'ok', val: undefined })
    })
};

export const stdin = {
    getStdin: () => ({
        read: () => ({ tag: 'ok', val: new Uint8Array(0) })
    })
};

export const stdout = {
    getStdout: () => ({
        write: (data) => {
            console.log(new TextDecoder().decode(data));
            return { tag: 'ok', val: BigInt(data.length) };
        },
        blockingFlush: () => ({ tag: 'ok', val: undefined })
    })
};
