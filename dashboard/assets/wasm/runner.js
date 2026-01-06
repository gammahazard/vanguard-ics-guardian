// ============================================================
// runner.js - wasm guest runner for browser
// ============================================================
//
// this module provides a simple api for the leptos dashboard to run
// the wasm guest component with specified security policies.
//
// usage:
//   import { runGuest, setLogCallback } from './runner.js';
//   setLogCallback((msg) => addToConsole(msg));
//   const result = await runGuest({ allowSensor: true, allowNetwork: false });
//
// related files:
// - malicious_driver.js: jco-generated wasm bindings
// - sensor-fs.js: filesystem shim with policy enforcement
// - sensor-net.js: network shim with data diode
// - sensor-utils.js: logging shim with callback support
// - index.html: loads this module and exposes window.wasmGuest api
// - lib.rs: leptos rust code that calls window.wasmGuest via wasm_bindgen
// ============================================================

import { setLogCallback as setUtilsLogCallback } from './shims/sensor-utils.js';
import { setPolicy as setFsPolicy } from './shims/sensor-fs.js';
import { setPolicy as setNetPolicy } from './shims/sensor-net.js';

// store for captured logs
let capturedLogs = [];

// set the log callback to capture all guest output
export function setLogCallback(callback) {
    setUtilsLogCallback(callback);
}

// run the guest component with specified policy
export async function runGuest(options = {}) {
    const { allowSensor = true, allowNetwork = false } = options;

    // reset logs
    capturedLogs = [];

    // configure policies before running
    setFsPolicy(allowSensor);
    setNetPolicy(allowNetwork);

    // set up log capture
    setUtilsLogCallback((msg) => {
        capturedLogs.push(msg);
    });

    try {
        // dynamically import and run the component
        // this avoids the preview2-shim import issue by catching errors
        const { run } = await import('./malicious_driver.js');
        await run();

        return {
            success: true,
            logs: capturedLogs,
            error: null
        };
    } catch (error) {
        return {
            success: false,
            logs: capturedLogs,
            error: error.message
        };
    }
}

// export policy setters for direct access if needed
export { setFsPolicy, setNetPolicy };
