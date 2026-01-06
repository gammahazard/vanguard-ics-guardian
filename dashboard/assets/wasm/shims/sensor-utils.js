// ============================================================
// sensor-utils.js - browser shim for logging capability
// ============================================================
//
// provides the vanguard:containment/sensor-utils interface for browser.
// captures log messages from the guest for display in the dashboard.
// the dashboard sets a callback before running to capture output.
//
// related files:
// - runner.js: sets logCallback before running guest
// - sensor-fs.js: filesystem capability shim
// - sensor-net.js: network capability shim (data diode)
// - malicious_driver.js: imports log from this module
// ============================================================

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
