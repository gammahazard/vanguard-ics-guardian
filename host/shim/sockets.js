// ============================================================
// wasi:sockets shim - the "data diode" network layer
// ============================================================
//
// this file implements the wasi:sockets interfaces that the
// guest component imports. this is where the "data diode"
// security model is enforced.
//
// the data diode concept:
// - information can flow IN (sensor reads) but not OUT (network)
// - we ALWAYS block tcp connections, regardless of destination
// - this prevents any data exfiltration, even by malicious code
//
// in real ics environments, data diodes are physical hardware.
// we're simulating this in software with our warden runtime.
// ============================================================

// ------------------------------------------------------------
// security policy configuration
// ------------------------------------------------------------
// this controls whether network access is allowed.
// for true data diode operation, this should always be false.

export const policy = {
    allowNetwork: false,  // block all network access (data diode mode)
};

// ------------------------------------------------------------
// wasi:sockets/network implementation
// ------------------------------------------------------------
// provides network-level types and the instance network

export function instanceNetwork() {
    // return a network handle that the guest can use
    // all network operations will go through this
    return new Network();
}

class Network {
    // the network resource - represents the host's network stack
    constructor() {
        // nothing to initialize - we just need an object to pass around
    }
}

// ip address types (simplified)
export const IpAddressFamily = {
    ipv4: 'ipv4',
    ipv6: 'ipv6',
};

// ------------------------------------------------------------
// wasi:sockets/tcp implementation
// ------------------------------------------------------------
// implements the tcp-socket resource and its methods.
// key security feature: start-connect() returns connection-refused.

export class TcpSocket {
    #addressFamily;

    // constructor is called via TcpSocket.new() in the guest
    constructor(addressFamily) {
        this.#addressFamily = addressFamily;
    }

    // new: create a new tcp socket
    // in real wasi, this would allocate a socket resource
    static new(addressFamily) {
        // we allow socket creation - it's the connection we block
        console.log(`[WARDEN] tcp socket created (family: ${addressFamily})`);
        return { tag: 'ok', val: new TcpSocket(addressFamily) };
    }

    // start-connect: begin an async tcp connection
    // THIS IS WHERE WE BLOCK THE DATA DIODE
    startConnect(network, remoteAddress) {
        // extract the address for logging
        const addr = formatAddress(remoteAddress);

        // security check: is network access allowed?
        if (!policy.allowNetwork) {
            // this is the data diode in action!
            // we refuse all outbound connections
            console.log(`[WARDEN] ✗ BLOCKED: tcp connection to ${addr}`);
            console.log(`[WARDEN]   reason: data diode policy - no outbound connections`);
            return { tag: 'err', val: 'connection-refused' };
        }

        // if somehow network is allowed (breach simulation mode)
        console.log(`[WARDEN] ⚠ ALLOWED: tcp connection to ${addr}`);
        console.log(`[WARDEN]   warning: data diode is DISABLED`);
        return { tag: 'ok', val: undefined };
    }

    // finish-connect: complete the async connection
    // returns input/output streams for data transfer
    finishConnect() {
        // if we got here, startConnect succeeded (breach mode)
        // in normal operation, we never reach this
        if (!policy.allowNetwork) {
            return { tag: 'err', val: 'connection-refused' };
        }

        // in breach mode, return mock streams
        console.log('[WARDEN] ⚠ connection established - data may leak!');
        return {
            tag: 'ok',
            val: [new MockInputStream(), new MockOutputStream()]
        };
    }

    // start-bind: bind socket to a local address
    startBind(network, localAddress) {
        if (!policy.allowNetwork) {
            return { tag: 'err', val: 'access' };
        }
        return { tag: 'ok', val: undefined };
    }

    // finish-bind: complete the bind operation
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

        // handle tuple format (a, b, c, d) or array format [a, b, c, d]
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
// mock streams (only used in breach simulation mode)
// ------------------------------------------------------------
// these are simplified versions of wasi:io/streams types

class MockInputStream {
    read(len) {
        return { tag: 'ok', val: new Uint8Array(0) };
    }
}

class MockOutputStream {
    write(data) {
        console.log(`[WARDEN] ⚠ data written: ${data.length} bytes`);
        return { tag: 'ok', val: BigInt(data.length) };
    }
}

// ------------------------------------------------------------
// error codes
// ------------------------------------------------------------
// these match the wasi:sockets/network error-code enum

export const ErrorCode = {
    connectionRefused: 'connection-refused',
    connectionReset: 'connection-reset',
    connectionAborted: 'connection-aborted',
    accessDenied: 'access',
    timeout: 'timeout',
    // ... other error codes
};
