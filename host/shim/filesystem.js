// ============================================================
// wasi:filesystem shim - the "data diode" filesystem layer
// ============================================================
//
// this file implements the wasi:filesystem interfaces that the
// guest component imports. we're acting as a mock host, providing
// controlled access to a virtual filesystem.
//
// key concepts:
// - preopens: directories the host grants to the guest at startup
// - descriptors: handles to files/directories (like file descriptors)
// - capability-based: guest can't access anything not explicitly granted
//
// our security policy:
// - when allowSensor=true: return mock sensor data
// - when allowSensor=false: deny all filesystem access
// ============================================================

// ------------------------------------------------------------
// security policy configuration
// ------------------------------------------------------------
// toggle this to control whether the guest can read files.
// in a real system, this would be set by the operator.

export const policy = {
  allowSensor: true,  // allow filesystem reads (default: enabled)
};

// ------------------------------------------------------------
// mock sensor data
// ------------------------------------------------------------
// this is what a "real" sensor on the oil rig would return.
// pressure, temperature, flow rate - typical ics telemetry.

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

// ------------------------------------------------------------
// wasi:filesystem/preopens implementation
// ------------------------------------------------------------
// get-directories() returns the directories the host has granted.
// in our case, we provide a single root directory.

export function getDirectories() {
  // return an array of [descriptor, path] tuples
  // the guest uses these as starting points for all file operations
  return [[new Descriptor('/'), '/']];
}

// ------------------------------------------------------------
// wasi:filesystem/types implementation
// ------------------------------------------------------------
// the descriptor class represents a file or directory handle.
// all file operations go through descriptors.

export class Descriptor {
  // private path this descriptor points to
  #path;
  
  constructor(path) {
    this.#path = path;
  }
  
  // open-at: open a file relative to this directory descriptor
  // this is how wasi does "openat" style operations
  openAt(pathFlags, path, openFlags, descriptorFlags) {
    // security check: is filesystem access allowed?
    if (!policy.allowSensor) {
      // return an error result - access denied
      console.log('[WARDEN] ✗ filesystem access blocked by policy');
      return { tag: 'err', val: 'access' };
    }
    
    // construct the full path
    const fullPath = this.#path === '/' 
      ? '/' + path 
      : this.#path + '/' + path;
    
    console.log(`[WARDEN] ✓ filesystem access allowed: ${fullPath}`);
    
    // return a new descriptor for the opened file
    // in real wasi, this would do actual file operations
    return { tag: 'ok', val: new Descriptor(fullPath) };
  }
  
  // read: read bytes from this file descriptor
  // returns [data, eof-status] tuple
  read(length, offset) {
    // check if this is the sensor data file
    if (this.#path.includes('sensor_data.json')) {
      // convert our mock data to bytes
      const encoder = new TextEncoder();
      const data = encoder.encode(MOCK_SENSOR_DATA);
      
      console.log(`[WARDEN] → returning ${data.length} bytes of sensor data`);
      
      // return success with data and "ended" (eof) status
      return { tag: 'ok', val: [data, 'ended'] };
    }
    
    // file not found
    console.log(`[WARDEN] ✗ file not found: ${this.#path}`);
    return { tag: 'err', val: 'no-entry' };
  }
  
  // stat: get file metadata (type, size, timestamps)
  stat() {
    const encoder = new TextEncoder();
    const size = BigInt(encoder.encode(MOCK_SENSOR_DATA).length);
    
    return {
      tag: 'ok',
      val: {
        type: 'regular-file',
        linkCount: 1n,
        size: size,
        dataAccessTimestamp: { seconds: 0n, nanoseconds: 0 },
        dataModificationTimestamp: { seconds: 0n, nanoseconds: 0 },
        statusChangeTimestamp: { seconds: 0n, nanoseconds: 0 },
      }
    };
  }
  
  // stat-at: get metadata for a path relative to this descriptor
  statAt(pathFlags, path) {
    return this.stat();
  }
  
  // get-type: return descriptor type (file, directory, etc)
  getType() {
    if (this.#path === '/') {
      return 'directory';
    }
    return 'regular-file';
  }
  
  // read-via-stream: get an input stream for reading
  // (simplified implementation)
  readViaStream(offset) {
    // in a full implementation, this would return a wasi:io/streams input-stream
    throw 'not-implemented';
  }
}

// ------------------------------------------------------------
// error codes
// ------------------------------------------------------------
// these match the wasi:filesystem/types error-code enum

export const ErrorCode = {
  access: 'access',
  noEntry: 'no-entry',
  notDirectory: 'not-directory',
  isDirectory: 'is-directory',
  invalidSeek: 'invalid-seek',
  // ... other error codes
};

// convenience for other modules to check the policy
export function isPolicyAllowed() {
  return policy.allowSensor;
}
