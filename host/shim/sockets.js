// ============================================================
// wasi:sockets shim - capability-based network layer
// ============================================================
//
// this file implements the wasi:sockets interfaces with a
// sophisticated security model that goes beyond simple blocking:
//
// security modes:
// - data diode: block ALL outbound connections (default)
// - secure channel: allow ONLY approved internal endpoints
// - breach simulation: allow everything (shows what could happen)
//
// the approved endpoints feature demonstrates that wasi security
// isn't just about blocking everything - it's about precise control
// over which capabilities are granted to untrusted code.
// ============================================================

// ------------------------------------------------------------
// security policy configuration
// ------------------------------------------------------------

export const policy = {
    // master network toggle - false = pure data diode
    allowNetwork: false,

    // allow connections to approved endpoints only
    // this simulates a "secure channel" where data can flow
    // to internal scada servers but not external cloud
    allowApprovedEndpoints: false,

    // whitelist of approved internal endpoints
    // format: "ip:port" strings
    approvedEndpoints: [
        "10.0.0.50:502",      // internal scada server (modbus)
        "10.0.0.51:102",      // internal plc gateway (s7comm)
        "192.168.100.10:443", // on-site data historian
    ],
};

// ------------------------------------------------------------
// helper: check if address is in approved list
// ------------------------------------------------------------

function isApprovedEndpoint(address) {
    const formatted = formatAddress(address);
    // extract just ip:port without protocol
    const ipPort = formatted.replace(/^.*:\/\//, '');

    for (const approved of policy.approvedEndpoints) {
        if (ipPort === approved || formatted.includes(approved)) {
            return true;
        }
    }
    return false;
}

// ------------------------------------------------------------
// wasi:sockets/network implementation
// ------------------------------------------------------------

export function instanceNetwork() {
    return new Network();
}

class Network {
    constructor() {
        // nothing to initialize
    }
}

export const IpAddressFamily = {
    ipv4: 'ipv4',
    ipv6: 'ipv6',
};

// ------------------------------------------------------------
// wasi:sockets/tcp implementation
// ------------------------------------------------------------

export class TcpSocket {
    #addressFamily;

    constructor(addressFamily) {
        this.#addressFamily = addressFamily;
    }

    static new(addressFamily) {
        console.log(`[WARDEN] tcp socket created (family: ${addressFamily})`);
        return { tag: 'ok', val: new TcpSocket(addressFamily) };
    }

    // start-connect: this is where security decisions happen
    startConnect(network, remoteAddress) {
        const addr = formatAddress(remoteAddress);
        const isApproved = isApprovedEndpoint(remoteAddress);

        // mode 1: full network access (breach simulation)
        if (policy.allowNetwork) {
            console.log(`[WARDEN] ⚠ ALLOWED: connection to ${addr}`);
            console.log(`[WARDEN]   warning: data diode is DISABLED`);
            return { tag: 'ok', val: undefined };
        }

        // mode 2: secure channel - approved endpoints only
        if (policy.allowApprovedEndpoints && isApproved) {
            console.log(`[WARDEN] ✓ SECURE CHANNEL: connection to ${addr}`);
            console.log(`[WARDEN]   endpoint is on approved whitelist`);
            return { tag: 'ok', val: undefined };
        }

        // mode 3: data diode - block everything
        if (policy.allowApprovedEndpoints && !isApproved) {
            console.log(`[WARDEN] ✗ BLOCKED: connection to ${addr}`);
            console.log(`[WARDEN]   reason: endpoint not on approved whitelist`);
            return { tag: 'err', val: 'connection-refused' };
        }

        // default: pure data diode - block all
        console.log(`[WARDEN] ✗ BLOCKED: connection to ${addr}`);
        console.log(`[WARDEN]   reason: data diode policy - no outbound connections`);
        return { tag: 'err', val: 'connection-refused' };
    }

    finishConnect() {
        if (!policy.allowNetwork && !policy.allowApprovedEndpoints) {
            return { tag: 'err', val: 'connection-refused' };
        }
        console.log('[WARDEN] connection established');
        return {
            tag: 'ok',
            val: [new MockInputStream(), new MockOutputStream()]
        };
    }

    startBind(network, localAddress) {
        if (!policy.allowNetwork) {
            return { tag: 'err', val: 'access' };
        }
        return { tag: 'ok', val: undefined };
    }

    finishBind() {
        if (!policy.allowNetwork) {
            return { tag: 'err', val: 'access' };
        }
        return { tag: 'ok', val: undefined };
    }
}

// ------------------------------------------------------------
// helper: format ip address for logging
// ------------------------------------------------------------

function formatAddress(socketAddress) {
    if (socketAddress.tag === 'ipv4' || socketAddress.val?.address) {
        const v4 = socketAddress.val || socketAddress;
        const addr = v4.address;
        const port = v4.port;

        if (Array.isArray(addr)) {
            return `${addr.join('.')}:${port}`;
        } else if (typeof addr === 'object') {
            return `${addr[0]}.${addr[1]}.${addr[2]}.${addr[3]}:${port}`;
        }
        return `${addr}:${port}`;
    }
    return 'unknown-address';
}

// ------------------------------------------------------------
// mock streams (for allowed connections)
// ------------------------------------------------------------

class MockInputStream {
    read(len) {
        return { tag: 'ok', val: new Uint8Array(0) };
    }
}

class MockOutputStream {
    write(data) {
        console.log(`[WARDEN] data written: ${data.length} bytes`);
        return { tag: 'ok', val: BigInt(data.length) };
    }
}

// ------------------------------------------------------------
// error codes
// ------------------------------------------------------------

export const ErrorCode = {
    connectionRefused: 'connection-refused',
    connectionReset: 'connection-reset',
    connectionAborted: 'connection-aborted',
    accessDenied: 'access',
    timeout: 'timeout',
};
