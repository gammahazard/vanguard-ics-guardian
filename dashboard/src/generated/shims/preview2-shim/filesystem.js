// ============================================================
// Minimal browser-compatible preview2-shim stubs for filesystem
// ============================================================
// Our guest uses our custom sensor-fs interface, not standard WASI filesystem.
// These stubs just prevent import errors.

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
