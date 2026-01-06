// ============================================================
// sensor-utils shim for browser - logging capability
// ============================================================
// This captures log messages from the guest for display in the dashboard.
// The dashboard will set the logCallback before running the guest.

let logCallback = null;

export function setLogCallback(callback) {
    logCallback = callback;
}

export function log(msg) {
    if (logCallback) {
        logCallback(msg);
    } else {
        console.log('[GUEST]', msg);
    }
}
