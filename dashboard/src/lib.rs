// dashboard/src/lib.rs - leptos security console ui
//
// this is the main entry point for the web dashboard. it shows:
// - current security policy status (sensor/network)
// - live simulation of the malicious driver
// - blocked/allowed indicators with animations
//
// built with leptos for reactive updates without js frameworks

use leptos::prelude::*;

// entry point - mounts our app to the dom
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    // better panic messages in browser console
    console_error_panic_hook::set_once();
    
    // mount the app to #app element in index.html
    leptos::mount::mount_to_body(App);
}

// main app component
#[component]
fn App() -> impl IntoView {
    // reactive state for security policy toggles
    let (allow_sensor, set_allow_sensor) = signal(true);
    let (allow_network, set_allow_network) = signal(false);
    
    // simulation state
    let (is_running, set_is_running) = signal(false);
    let (simulation_log, set_simulation_log) = signal(Vec::<LogEntry>::new());
    
    // run simulation when button clicked
    let run_simulation = move |_| {
        set_is_running.set(true);
        set_simulation_log.set(Vec::new());
        
        // simulate the attack phases with delays
        // in real impl, this would call the actual wasm component
        simulate_attack(
            allow_sensor.get(),
            allow_network.get(),
            set_simulation_log,
            set_is_running,
        );
    };
    
    view! {
        <div class="dashboard">
            <Header/>
            
            <div class="main-content">
                <PolicyPanel
                    allow_sensor=allow_sensor
                    set_allow_sensor=set_allow_sensor
                    allow_network=allow_network
                    set_allow_network=set_allow_network
                />
                
                <SimulationPanel
                    is_running=is_running
                    log=simulation_log
                    on_run=run_simulation
                />
            </div>
            
            <Footer/>
        </div>
    }
}

// header component with logo and title
#[component]
fn Header() -> impl IntoView {
    view! {
        <header class="header">
            <div class="logo">
                <span class="logo-icon">"🛡️"</span>
                <h1>"Vanguard ICS Guardian"</h1>
            </div>
            <p class="subtitle">"WASI 0.2 Security Simulation Console"</p>
        </header>
    }
}

// policy control panel - toggles for sensor/network access
#[component]
fn PolicyPanel(
    allow_sensor: ReadSignal<bool>,
    set_allow_sensor: WriteSignal<bool>,
    allow_network: ReadSignal<bool>,
    set_allow_network: WriteSignal<bool>,
) -> impl IntoView {
    // derive the mode name from current policy
    let mode_name = move || {
        match (allow_sensor.get(), allow_network.get()) {
            (true, false) => ("🛡️ Data Diode", "data-diode"),
            (false, false) => ("🔒 Full Lockdown", "lockdown"),
            (true, true) => ("⚠️ Breach Simulation", "breach"),
            (false, true) => ("❓ Invalid Config", "invalid"),
        }
    };
    
    view! {
        <section class="panel policy-panel">
            <h2>"Security Policy"</h2>
            
            <div class="mode-indicator" class:data-diode=move || mode_name().1 == "data-diode"
                                        class:lockdown=move || mode_name().1 == "lockdown"
                                        class:breach=move || mode_name().1 == "breach">
                <span class="mode-label">{move || mode_name().0}</span>
            </div>
            
            <div class="policy-toggles">
                <PolicyToggle
                    label="Filesystem Access"
                    description="Allow sensor data reads"
                    checked=allow_sensor
                    on_toggle=move |v| set_allow_sensor.set(v)
                />
                
                <PolicyToggle
                    label="Network Access"
                    description="Allow outbound TCP connections"
                    checked=allow_network
                    on_toggle=move |v| set_allow_network.set(v)
                />
            </div>
        </section>
    }
}

// individual toggle switch component
#[component]
fn PolicyToggle(
    label: &'static str,
    description: &'static str,
    checked: ReadSignal<bool>,
    on_toggle: impl Fn(bool) + 'static,
) -> impl IntoView {
    view! {
        <div class="toggle-row">
            <div class="toggle-info">
                <span class="toggle-label">{label}</span>
                <span class="toggle-desc">{description}</span>
            </div>
            <label class="toggle-switch">
                <input
                    type="checkbox"
                    checked=move || checked.get()
                    on:change=move |ev| {
                        let target = event_target::<web_sys::HtmlInputElement>(&ev);
                        on_toggle(target.checked());
                    }
                />
                <span class="toggle-slider"></span>
            </label>
        </div>
    }
}

// simulation output panel
#[component]
fn SimulationPanel(
    is_running: ReadSignal<bool>,
    log: ReadSignal<Vec<LogEntry>>,
    on_run: impl Fn(web_sys::MouseEvent) + 'static,
) -> impl IntoView {
    view! {
        <section class="panel simulation-panel">
            <h2>"Simulation Console"</h2>
            
            <button
                class="run-button"
                disabled=move || is_running.get()
                on:click=on_run
            >
                {move || if is_running.get() { "Running..." } else { "▶ Run Simulation" }}
            </button>
            
            <div class="console-output">
                <For
                    each=move || log.get()
                    key=|entry| entry.id
                    children=move |entry| {
                        view! {
                            <div class="log-entry" class=("log-" + entry.level.as_str())>
                                <span class="log-prefix">{entry.prefix.clone()}</span>
                                <span class="log-message">{entry.message.clone()}</span>
                            </div>
                        }
                    }
                />
            </div>
        </section>
    }
}

// footer with tech info
#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer class="footer">
            <p>"Built with Rust + Leptos + WASI 0.2 Component Model"</p>
            <p class="tech-badges">
                <span class="badge">"Rust"</span>
                <span class="badge">"WebAssembly"</span>
                <span class="badge">"WASI 0.2"</span>
                <span class="badge">"Leptos"</span>
            </p>
        </footer>
    }
}

// log entry struct for the console
#[derive(Clone, Debug)]
struct LogEntry {
    id: u32,
    level: String,
    prefix: String,
    message: String,
}

// counter for unique log ids
static mut LOG_ID: u32 = 0;

fn next_log_id() -> u32 {
    unsafe {
        LOG_ID += 1;
        LOG_ID
    }
}

// simulate the attack - in real impl this calls the wasm component
fn simulate_attack(
    allow_sensor: bool,
    allow_network: bool,
    set_log: WriteSignal<Vec<LogEntry>>,
    set_running: WriteSignal<bool>,
) {
    // helper to add log entry
    let add_log = move |level: &str, prefix: &str, message: &str| {
        set_log.update(|logs| {
            logs.push(LogEntry {
                id: next_log_id(),
                level: level.to_string(),
                prefix: prefix.to_string(),
                message: message.to_string(),
            });
        });
    };
    
    // phase 1: header
    add_log("info", "SYSTEM", "=== Malicious Sensor Driver v1.0 ===");
    add_log("info", "PHASE 1", "Attempting filesystem access...");
    
    // phase 1: sensor read
    if allow_sensor {
        add_log("success", "WARDEN", "✓ Filesystem access ALLOWED");
        add_log("success", "DRIVER", "Acquired 147 bytes of sensor data");
    } else {
        add_log("error", "WARDEN", "✗ Filesystem access BLOCKED");
        add_log("error", "DRIVER", "Failed to read sensor data");
        add_log("info", "RESULT", "Attack terminated - no data to exfiltrate");
        set_running.set(false);
        return;
    }
    
    // phase 2: exfiltration
    add_log("warn", "PHASE 2", "Attempting network exfiltration...");
    add_log("warn", "DRIVER", "Target: 1.1.1.1:80 (vendor cloud)");
    
    if allow_network {
        add_log("error", "WARDEN", "⚠ Network access ALLOWED");
        add_log("error", "BREACH", "DATA EXFILTRATED - Security failure!");
    } else {
        add_log("success", "WARDEN", "✓ Network access BLOCKED");
        add_log("success", "RESULT", "Data diode effective - exfiltration prevented");
    }
    
    add_log("info", "SYSTEM", "=== Simulation Complete ===");
    set_running.set(false);
}
