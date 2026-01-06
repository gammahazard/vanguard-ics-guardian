// ============================================================
// WASM Guest Runner for Browser
// ============================================================
// This module provides a simple API for the Leptos dashboard to run
// the WASM guest component with specified security policies.
//
// Usage:
//   import { runGuest, setLogCallback } from './runner.js';
//   setLogCallback((msg) => addToConsole(msg));
//   const result = await runGuest({ allowSensor: true, allowNetwork: false });

import { setLogCallback as setUtilsLogCallback } from './shims/sensor-utils.js';
import { setPolicy as setFsPolicy } from './shims/sensor-fs.js';
import { setPolicy as setNetPolicy } from './shims/sensor-net.js';

// Store for captured logs
let capturedLogs = [];

// Set the log callback to capture all guest output
export function setLogCallback(callback) {
    setUtilsLogCallback(callback);
}

// Run the guest component with specified policy
export async function runGuest(options = {}) {
    const { allowSensor = true, allowNetwork = false } = options;

    // Reset logs
    capturedLogs = [];

    // Configure policies before running
    setFsPolicy(allowSensor);
    setNetPolicy(allowNetwork);

    // Set up log capture
    setUtilsLogCallback((msg) => {
        capturedLogs.push(msg);
    });

    try {
        // Dynamically import and run the component
        // This avoids the preview2-shim import issue by catching errors
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

// Export policy setters for direct access if needed
export { setFsPolicy, setNetPolicy };
