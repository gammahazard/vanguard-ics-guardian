// ============================================================
// sensor-net.js - browser shim for network capability (data diode)
// ============================================================
//
// provides the vanguard:containment/sensor-net interface for browser.
// this is the data diode - blocks network access by default.
// policy is controlled by the dashboard via setPolicy().
//
// related files:
// - runner.js: calls setPolicy() before running guest
// - sensor-fs.js: filesystem capability shim
// - sensor-utils.js: logging capability shim
// - malicious_driver.js: imports sendTelemetry from this module
//
// note: jco expects shims to return raw values and throw exceptions,
// not { tag: 'ok/err', val: ... } result objects.
// ============================================================

// policy state - controlled by dashboard (default: block network)
let allowNetwork = false;

export function setPolicy(allow) {
    console.log('[SHIM] setPolicy (network):', allow);
    allowNetwork = allow;
}

export function sendTelemetry(data) {
    console.log('[SHIM] sendTelemetry called with', data.length, 'bytes');

    if (!allowNetwork) {
        console.log('[SHIM] network BLOCKED - data diode engaged');
        // throw for error case - jco will catch and convert to err result
        throw Object.assign(new Error('connection-refused: network access blocked by data diode'), {
            payload: 'connection-refused: network access blocked by data diode'
        });
    }

    // network allowed (breach simulation)
    console.log('[SHIM] network ALLOWED - data exfiltrated!');
    // return raw value - jco will wrap in ok result
    return data.length;
}
