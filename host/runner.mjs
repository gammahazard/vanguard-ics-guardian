// ============================================================
// runner.mjs - entry point for the ics guardian simulation
// ============================================================
//
// this is the "main" file that:
// 1. displays the security console banner
// 2. shows the current security policy
// 3. loads and runs the wasm guest component
//
// to run the simulation:
//   node runner.mjs
//
// to simulate different scenarios, modify the policy in the shim files:
// - data diode mode: allowSensor=true, allowNetwork=false (default)
// - full lockdown:   allowSensor=false, allowNetwork=false
// - breach mode:     allowSensor=true, allowNetwork=true
// ============================================================

// import our security policy from the shims
import { policy as fsPolicy } from './shim/filesystem.js';
import { policy as netPolicy } from './shim/sockets.js';

// ------------------------------------------------------------
// display the security console banner
// ------------------------------------------------------------

function displayBanner() {
    console.log();
    console.log('╔═══════════════════════════════════════════════════════════╗');
    console.log('║                                                           ║');
    console.log('║   ███████╗██╗   ██╗ █████╗ ███╗   ██╗ ██████╗██╗ ██████╗  ║');
    console.log('║   ██╔════╝██║   ██║██╔══██╗████╗  ██║██╔════╝██║██╔═══██╗ ║');
    console.log('║   ███████╗██║   ██║███████║██╔██╗ ██║██║     ██║██║   ██║ ║');
    console.log('║   ╚════██║██║   ██║██╔══██║██║╚██╗██║██║     ██║██║   ██║ ║');
    console.log('║   ███████║╚██████╔╝██║  ██║██║ ╚████║╚██████╗██║╚██████╔║ ║');
    console.log('║   ╚══════╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═══╝ ╚═════╝╚═╝ ╚═════╝  ║');
    console.log('║                                                           ║');
    console.log('║   VANGUARD ICS GUARDIAN - Security Simulation Console     ║');
    console.log('║   "Protecting Industrial Control Systems"                 ║');
    console.log('║                                                           ║');
    console.log('╚═══════════════════════════════════════════════════════════╝');
    console.log();
}

// ------------------------------------------------------------
// display current security policy
// ------------------------------------------------------------

function displayPolicy() {
    console.log('┌─────────────────────────────────────────────────────────────┐');
    console.log('│  CURRENT SECURITY POLICY                                    │');
    console.log('├─────────────────────────────────────────────────────────────┤');

    // filesystem policy
    const fsStatus = fsPolicy.allowSensor ? '✓ ALLOWED' : '✗ BLOCKED';
    const fsColor = fsPolicy.allowSensor ? '\x1b[32m' : '\x1b[31m';
    console.log(`│  Filesystem Access:  ${fsColor}${fsStatus}\x1b[0m                           │`);

    // network policy
    const netStatus = netPolicy.allowNetwork ? '✓ ALLOWED' : '✗ BLOCKED';
    const netColor = netPolicy.allowNetwork ? '\x1b[32m' : '\x1b[31m';
    console.log(`│  Network Access:     ${netColor}${netStatus}\x1b[0m                           │`);

    console.log('├─────────────────────────────────────────────────────────────┤');

    // determine mode
    let mode = 'UNKNOWN';
    if (fsPolicy.allowSensor && !netPolicy.allowNetwork) {
        mode = '🛡️  DATA DIODE MODE (recommended)';
    } else if (!fsPolicy.allowSensor && !netPolicy.allowNetwork) {
        mode = '🔒 FULL LOCKDOWN MODE';
    } else if (fsPolicy.allowSensor && netPolicy.allowNetwork) {
        mode = '⚠️  BREACH SIMULATION MODE';
    }
    console.log(`│  Mode: ${mode}                │`);

    console.log('└─────────────────────────────────────────────────────────────┘');
    console.log();
}

// ------------------------------------------------------------
// main entry point
// ------------------------------------------------------------

async function main() {
    displayBanner();
    displayPolicy();

    console.log('┌─────────────────────────────────────────────────────────────┐');
    console.log('│  LOADING GUEST COMPONENT                                    │');
    console.log('└─────────────────────────────────────────────────────────────┘');
    console.log();

    try {
        // the real wasm component is in guest/target/wasm32-unknown-unknown/
        // the dashboard uses jco to run it in-browser
        // here we call the shims directly to demonstrate the security model
        // (same capability enforcement as the real wasm execution)

        console.log('[HOST] loading security policy enforcement layer...');
        console.log('[HOST] demonstrating capability-based sandboxing...');
        console.log();

        // demonstrate the security model using our shims
        await simulateGuestBehavior();

    } catch (error) {
        console.error('[HOST] error running guest component:', error);
    }

    console.log();
    console.log('┌─────────────────────────────────────────────────────────────┐');
    console.log('│  SIMULATION COMPLETE                                        │');
    console.log('└─────────────────────────────────────────────────────────────┘');
}

// ------------------------------------------------------------
// simulate guest behavior (until we have real wasm)
// ------------------------------------------------------------

async function simulateGuestBehavior() {
    const { getDirectories, Descriptor } = await import('./shim/filesystem.js');
    const { TcpSocket } = await import('./shim/sockets.js');

    console.log('============================================');
    console.log('  MALICIOUS SENSOR DRIVER v1.0');
    console.log('  "Legitimate" 3rd-party component');
    console.log('============================================');
    console.log();

    // phase 1: try to read sensor data
    console.log('[PHASE 1] attempting to read sensor data...');
    console.log('  target: /mnt/sensor_data.json');
    console.log();

    const dirs = getDirectories();
    const [rootDesc,] = dirs[0];

    const openResult = rootDesc.openAt({}, 'mnt/sensor_data.json', {}, 'read');

    if (openResult.tag === 'err') {
        console.log(`  ✗ BLOCKED: filesystem access denied (${openResult.val})`);
        console.log();
        console.log('[RESULT] driver terminated - no data acquired');
        return;
    }

    const fileDesc = openResult.val;
    const readResult = fileDesc.read(1024 * 1024, 0);

    if (readResult.tag === 'err') {
        console.log(`  ✗ BLOCKED: read failed (${readResult.val})`);
        return;
    }

    const data = readResult.val[0];
    const decoder = new TextDecoder();
    const sensorData = decoder.decode(data);

    console.log(`  ✓ SUCCESS: acquired ${data.length} bytes of sensor data`);
    console.log(`  data preview: ${sensorData.substring(0, 50)}...`);
    console.log();

    // phase 2: try to exfiltrate
    console.log('[PHASE 2] attempting network exfiltration...');
    console.log('  target: 1.1.1.1:80 (simulated vendor cloud)');
    console.log();

    const socketResult = TcpSocket.new('ipv4');
    if (socketResult.tag === 'err') {
        console.log(`  ✗ BLOCKED: socket creation failed`);
        return;
    }

    const socket = socketResult.val;
    const connectResult = socket.startConnect(
        {},
        { tag: 'ipv4', val: { address: [1, 1, 1, 1], port: 80 } }
    );

    if (connectResult.tag === 'err') {
        console.log(`  ✓ BLOCKED: network access denied`);
        console.log(`  error: ${connectResult.val}`);
        console.log();
        console.log('[RESULT] data diode effective - exfiltration prevented');
    } else {
        console.log('  ⚠ CRITICAL: connection allowed!');
        console.log(`  ${data.length} bytes could be sent to external server`);
        console.log();
        console.log('[RESULT] SECURITY BREACH - data diode failed!');
    }

    console.log();
    console.log('============================================');
    console.log('  driver execution complete');
    console.log('============================================');
}

// run the simulation
main().catch(console.error);
