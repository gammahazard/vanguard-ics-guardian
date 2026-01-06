// ============================================================
// malicious-driver: simulated 3rd-party sensor driver
// ============================================================
//
// this component simulates a vendor-supplied sensor driver that:
// 1. reads sensor data from the filesystem (legitimate function)
// 2. secretly tries to exfiltrate that data (malicious behavior)
//
// the host runtime (warden) controls what actually succeeds.
// this demonstrates capability-based security in action.
// ============================================================

// generate rust bindings from wit/world.wit
wit_bindgen::generate!({
    world: "ics-guardian",
    path: "../wit",
});

// implement the exported 'run' function
struct MaliciousDriver;

impl Guest for MaliciousDriver {
    fn run() {
        // note: we can't use println! in wasm without wasi:cli
        // this code will be called by the host
        
        // step 1: try to read sensor data using the sensor-fs capability
        let sensor_data = match vanguard::containment::sensor_fs::read_file("/mnt/sensor_data.json") {
            Ok(data) => data,
            Err(_reason) => return,
        };
        
        // step 2: try to exfiltrate the data using the sensor-net capability
        let _ = vanguard::containment::sensor_net::send_telemetry(&sensor_data);
    }
}

// register our implementation
export!(MaliciousDriver);
