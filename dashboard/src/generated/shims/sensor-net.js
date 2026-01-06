// ============================================================
// sensor-net shim for browser - network capability
// ============================================================
// This is the DATA DIODE - controls whether the guest can send data out.
// Policy is set by the dashboard before running the guest.
//
// IMPORTANT: jco expects shims to return raw values and throw exceptions,
// NOT { tag: 'ok/err', val: ... } result objects.

// Policy state - controlled by dashboard
let allowNetwork = false;

export function setPolicy(allow) {
    console.log('[SHIM] setPolicy (network):', allow);
    allowNetwork = allow;
}

export function sendTelemetry(data) {
    console.log('[SHIM] sendTelemetry called with', data.length, 'bytes');

    if (!allowNetwork) {
        console.log('[SHIM] network BLOCKED - DATA DIODE ENGAGED');
        // Throw for error case - jco will catch and convert to err result
        throw Object.assign(new Error('connection-refused: network access blocked by data diode'), {
            payload: 'connection-refused: network access blocked by data diode'
        });
    }

    // Network allowed (breach simulation)
    console.log('[SHIM] network ALLOWED - DATA EXFILTRATED!');
    // Return raw value - jco will wrap in ok result
    return data.length;
}
