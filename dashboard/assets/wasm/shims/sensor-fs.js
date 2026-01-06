// ============================================================
// sensor-fs.js - browser shim for filesystem capability
// ============================================================
//
// provides the vanguard:containment/sensor-fs interface for browser.
// this file is loaded by malicious_driver.js (jco-transpiled wasm).
// policy is controlled by the dashboard via setPolicy().
//
// related files:
// - runner.js: calls setPolicy() before running guest
// - sensor-net.js: network capability shim (data diode)
// - sensor-utils.js: logging capability shim
// - malicious_driver.js: imports readFile from this module
//
// note: jco expects shims to return raw values and throw exceptions,
// not { tag: 'ok/err', val: ... } result objects.
// ============================================================

// mock sensor data (same as host shim in host/shim/filesystem.js)
const MOCK_SENSOR_DATA = JSON.stringify({
    pressure_psi: 2847.3,
    temperature_c: 67.8,
    flow_rate_bpm: 1250.0,
    vibration_hz: 42.1,
    well_id: "PLATFORM-7-WELL-03",
    timestamp: new Date().toISOString(),
    status: "nominal",
    alerts: []
}, null, 2);

// policy state - controlled by dashboard
let allowSensor = true;

export function setPolicy(allow) {
    console.log('[SHIM] setPolicy (sensor):', allow);
    allowSensor = allow;
}

export function readFile(path) {
    console.log('[SHIM] readFile called with path:', path);

    if (!allowSensor) {
        console.log('[SHIM] access DENIED - throwing error');
        // throw for error case - jco will catch and convert to err result
        throw Object.assign(new Error('access-denied: filesystem access blocked by policy'), {
            payload: 'access-denied: filesystem access blocked by policy'
        });
    }

    // check if this is the sensor data file
    if (path.includes('sensor_data.json')) {
        console.log('[SHIM] returning sensor data, length:', MOCK_SENSOR_DATA.length);
        // return raw string - jco will wrap in ok result
        return MOCK_SENSOR_DATA;
    }

    console.log('[SHIM] file not found - throwing error');
    throw Object.assign(new Error('no-entry: file not found'), {
        payload: 'no-entry: file not found'
    });
}
