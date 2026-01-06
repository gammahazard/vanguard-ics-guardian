// ============================================================
// filesystem.js - minimal browser-compatible wasi:filesystem stubs
// ============================================================
//
// provides minimal implementations of wasi:filesystem interfaces for browser.
// our guest uses the custom sensor-fs interface, not standard wasi filesystem.
// these stubs just prevent import errors from jco-generated bindings.
//
// related files:
// - cli.js: wasi:cli stubs
// - io.js: wasi:io stubs (streams, error)
// - sensor-fs.js: actual filesystem shim with policy enforcement
// - malicious_driver.js: imports from this module
// ============================================================

class Descriptor {
    constructor(path) {
        this.path = path;
    }

    openAt(pathFlags, path, openFlags, descriptorFlags) {
        return { tag: 'err', val: 'access' };
    }

    read(length, offset) {
        return { tag: 'err', val: 'no-entry' };
    }

    stat() {
        return {
            tag: 'ok',
            val: {
                type: 'regular-file',
                linkCount: 1n,
                size: 0n,
                dataAccessTimestamp: { seconds: 0n, nanoseconds: 0 },
                dataModificationTimestamp: { seconds: 0n, nanoseconds: 0 },
                statusChangeTimestamp: { seconds: 0n, nanoseconds: 0 },
            }
        };
    }

    getType() {
        return 'regular-file';
    }
}

export const preopens = {
    getDirectories: () => [[new Descriptor('/'), '/']]
};

export const types = {
    Descriptor,
    filesystemErrorCode: (err) => err
};
