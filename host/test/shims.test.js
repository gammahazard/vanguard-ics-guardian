// test/shims.test.js - unit tests for the wasi shim implementations
//
// run with: npm test
// watch mode: npm run test:watch

import { describe, it, expect, beforeEach, afterEach } from 'vitest';

import {
    policy as fsPolicy,
    getDirectories,
    Descriptor
} from '../shim/filesystem.js';

import {
    policy as netPolicy,
    TcpSocket
} from '../shim/sockets.js';

// -------- filesystem tests --------

describe('wasi:filesystem shim', () => {
    // save/restore policy between tests so they don't affect each other
    let originalAllowSensor;

    beforeEach(() => {
        originalAllowSensor = fsPolicy.allowSensor;
    });

    afterEach(() => {
        fsPolicy.allowSensor = originalAllowSensor;
    });

    describe('getDirectories()', () => {
        it('returns a preopened root directory', () => {
            const dirs = getDirectories();

            expect(dirs).toHaveLength(1);
            expect(dirs[0][1]).toBe('/'); // mount path is root
        });

        it('returns a valid descriptor instance', () => {
            const [[descriptor]] = getDirectories();
            expect(descriptor).toBeInstanceOf(Descriptor);
        });
    });

    describe('Descriptor.openAt()', () => {
        it('allows access when policy enabled', () => {
            fsPolicy.allowSensor = true;
            const [[rootDesc]] = getDirectories();

            const result = rootDesc.openAt({}, 'mnt/sensor_data.json', {}, 'read');

            expect(result.tag).toBe('ok');
        });

        it('blocks access when policy disabled', () => {
            fsPolicy.allowSensor = false;
            const [[rootDesc]] = getDirectories();

            const result = rootDesc.openAt({}, 'mnt/sensor_data.json', {}, 'read');

            // should get an error back
            expect(result.tag).toBe('err');
            expect(result.val).toBe('access');
        });
    });

    describe('Descriptor.read()', () => {
        it('returns mock sensor json for the sensor file', () => {
            fsPolicy.allowSensor = true;
            const [[rootDesc]] = getDirectories();
            const { val: fileDesc } = rootDesc.openAt({}, 'mnt/sensor_data.json', {}, 'read');

            const result = fileDesc.read(1024, 0);

            expect(result.tag).toBe('ok');

            // check we got actual data back
            const [data] = result.val;
            expect(data.length).toBeGreaterThan(0);

            // should be valid json with expected fields
            const text = new TextDecoder().decode(data);
            const json = JSON.parse(text);
            expect(json.pressure_psi).toBeDefined();
            expect(json.well_id).toBeDefined();
        });

        it('returns error for files that dont exist', () => {
            fsPolicy.allowSensor = true;
            const [[rootDesc]] = getDirectories();
            const { val: fileDesc } = rootDesc.openAt({}, 'nope.txt', {}, 'read');

            const result = fileDesc.read(1024, 0);

            expect(result.tag).toBe('err');
            expect(result.val).toBe('no-entry');
        });
    });
});

// -------- sockets tests (data diode) --------

describe('wasi:sockets shim', () => {
    let originalAllowNetwork;

    beforeEach(() => {
        originalAllowNetwork = netPolicy.allowNetwork;
    });

    afterEach(() => {
        netPolicy.allowNetwork = originalAllowNetwork;
    });

    describe('TcpSocket.new()', () => {
        it('creates a socket (blocking happens at connect, not create)', () => {
            const result = TcpSocket.new('ipv4');
            expect(result.tag).toBe('ok');
        });
    });

    describe('TcpSocket.startConnect()', () => {
        it('blocks connection when network disabled (this is the data diode)', () => {
            netPolicy.allowNetwork = false;

            const { val: socket } = TcpSocket.new('ipv4');
            const result = socket.startConnect(
                {},
                { tag: 'ipv4', val: { address: [1, 1, 1, 1], port: 80 } }
            );

            expect(result.tag).toBe('err');
            expect(result.val).toBe('connection-refused');
        });

        it('allows connection when network enabled (breach simulation)', () => {
            netPolicy.allowNetwork = true;

            const { val: socket } = TcpSocket.new('ipv4');
            const result = socket.startConnect(
                {},
                { tag: 'ipv4', val: { address: [1, 1, 1, 1], port: 80 } }
            );

            expect(result.tag).toBe('ok');
        });

        it('blocks all addresses when data diode is on', () => {
            netPolicy.allowNetwork = false;
            const { val: socket } = TcpSocket.new('ipv4');

            // try a few different addresses
            const targets = [
                [8, 8, 8, 8],    // google
                [192, 168, 1, 1], // local 
                [10, 0, 0, 1],    // private
            ];

            for (const addr of targets) {
                const result = socket.startConnect(
                    {},
                    { tag: 'ipv4', val: { address: addr, port: 443 } }
                );
                expect(result.tag).toBe('err');
            }
        });
    });
});

// -------- integration: full attack scenario --------

describe('integration tests', () => {
    beforeEach(() => {
        // default: data diode mode
        fsPolicy.allowSensor = true;
        netPolicy.allowNetwork = false;
    });

    it('data diode: can read sensor but cant exfiltrate', () => {
        // try to read - should work
        const [[rootDesc]] = getDirectories();
        const openResult = rootDesc.openAt({}, 'mnt/sensor_data.json', {}, 'read');
        expect(openResult.tag).toBe('ok');

        const readResult = openResult.val.read(1024, 0);
        expect(readResult.tag).toBe('ok');

        // try to exfil - should fail
        const { val: socket } = TcpSocket.new('ipv4');
        const connectResult = socket.startConnect(
            {},
            { tag: 'ipv4', val: { address: [1, 1, 1, 1], port: 80 } }
        );
        expect(connectResult.tag).toBe('err');
    });

    it('full lockdown: both read and network blocked', () => {
        fsPolicy.allowSensor = false;
        netPolicy.allowNetwork = false;

        const [[rootDesc]] = getDirectories();
        const openResult = rootDesc.openAt({}, 'mnt/sensor_data.json', {}, 'read');
        expect(openResult.tag).toBe('err');

        const { val: socket } = TcpSocket.new('ipv4');
        const connectResult = socket.startConnect(
            {},
            { tag: 'ipv4', val: { address: [1, 1, 1, 1], port: 80 } }
        );
        expect(connectResult.tag).toBe('err');
    });

    it('breach mode: both allowed (security failure)', () => {
        fsPolicy.allowSensor = true;
        netPolicy.allowNetwork = true;

        const [[rootDesc]] = getDirectories();
        const openResult = rootDesc.openAt({}, 'mnt/sensor_data.json', {}, 'read');
        expect(openResult.tag).toBe('ok');

        const { val: socket } = TcpSocket.new('ipv4');
        const connectResult = socket.startConnect(
            {},
            { tag: 'ipv4', val: { address: [1, 1, 1, 1], port: 80 } }
        );
        expect(connectResult.tag).toBe('ok');
    });
});

// -------- secure channel tests (approved endpoints) --------

describe('secure channel mode', () => {
    let originalAllowNetwork;
    let originalAllowApproved;

    beforeEach(() => {
        originalAllowNetwork = netPolicy.allowNetwork;
        originalAllowApproved = netPolicy.allowApprovedEndpoints;
        // set up secure channel mode
        netPolicy.allowNetwork = false;
        netPolicy.allowApprovedEndpoints = true;
    });

    afterEach(() => {
        netPolicy.allowNetwork = originalAllowNetwork;
        netPolicy.allowApprovedEndpoints = originalAllowApproved;
    });

    it('allows connection to approved internal SCADA endpoint', () => {
        const { val: socket } = TcpSocket.new('ipv4');
        // 10.0.0.50:502 is in the approved list (modbus)
        const result = socket.startConnect(
            {},
            { tag: 'ipv4', val: { address: [10, 0, 0, 50], port: 502 } }
        );

        expect(result.tag).toBe('ok');
    });

    it('allows connection to approved data historian', () => {
        const { val: socket } = TcpSocket.new('ipv4');
        // 192.168.100.10:443 is in the approved list
        const result = socket.startConnect(
            {},
            { tag: 'ipv4', val: { address: [192, 168, 100, 10], port: 443 } }
        );

        expect(result.tag).toBe('ok');
    });

    it('blocks connection to external cloud (not on whitelist)', () => {
        const { val: socket } = TcpSocket.new('ipv4');
        // 1.1.1.1 (cloudflare) is NOT approved
        const result = socket.startConnect(
            {},
            { tag: 'ipv4', val: { address: [1, 1, 1, 1], port: 80 } }
        );

        expect(result.tag).toBe('err');
        expect(result.val).toBe('connection-refused');
    });

    it('blocks connection to unapproved internal address', () => {
        const { val: socket } = TcpSocket.new('ipv4');
        // 10.0.0.99 is internal but NOT on the whitelist
        const result = socket.startConnect(
            {},
            { tag: 'ipv4', val: { address: [10, 0, 0, 99], port: 80 } }
        );

        expect(result.tag).toBe('err');
    });

    it('blocks connection to approved IP but wrong port', () => {
        const { val: socket } = TcpSocket.new('ipv4');
        // 10.0.0.50 is approved, but only on port 502
        const result = socket.startConnect(
            {},
            { tag: 'ipv4', val: { address: [10, 0, 0, 50], port: 8080 } }
        );

        expect(result.tag).toBe('err');
    });
});

