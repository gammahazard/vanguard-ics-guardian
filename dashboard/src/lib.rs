// dashboard/src/lib.rs - leptos security console ui
//
// this is the main entry point for the web dashboard. it shows:
// - the oil rig scenario with sensor data visualization
// - current security policy status (sensor/network)
// - live simulation of the malicious driver attack
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
    let (sensor_data, set_sensor_data) = signal(Option::<SensorData>::None);
    
    // run simulation when button clicked
    let run_simulation = move |_| {
        set_is_running.set(true);
        set_simulation_log.set(Vec::new());
        set_sensor_data.set(None);
        
        simulate_attack(
            allow_sensor.get(),
            allow_network.get(),
            set_simulation_log,
            set_is_running,
            set_sensor_data,
        );
    };
    
    view! {
        <div class="dashboard">
            <Header/>
            
            <ScenarioContext/>
            
            <div class="main-content">
                <div class="left-panel">
                    <PolicyPanel
                        allow_sensor=allow_sensor
                        set_allow_sensor=set_allow_sensor
                        allow_network=allow_network
                        set_allow_network=set_allow_network
                    />
                    
                    <SensorDataPanel data=sensor_data/>
                </div>
                
                <SimulationPanel
                    is_running=is_running
                    log=simulation_log
                    on_run=run_simulation
                />
            </div>
            
            <DeploymentPanel/>
            
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
            <p class="subtitle">"WASI 0.2 Capability-Based Security Demonstration"</p>
        </header>
    }
}

// scenario context - explains the oil rig situation
#[component]
fn ScenarioContext() -> impl IntoView {
    view! {
        <section class="scenario-context">
            <div class="scenario-header">
                <span class="oil-rig-icon">"🛢️"</span>
                <div>
                    <h2>"Platform 7 - Gulf of Mexico"</h2>
                    <p class="scenario-subtitle">"Offshore Oil Rig Control System"</p>
                </div>
            </div>
            
            <div class="scenario-diagram">
                <div class="diagram-node sensor">
                    <span class="node-icon">"📊"</span>
                    <span class="node-label">"Pressure Sensor"</span>
                </div>
                <div class="diagram-arrow">"→"</div>
                <div class="diagram-node driver">
                    <span class="node-icon">"⚙️"</span>
                    <span class="node-label">"3rd-Party Driver"</span>
                    <span class="node-warning">"(untrusted)"</span>
                </div>
                <div class="diagram-arrow">"→"</div>
                <div class="diagram-node diode">
                    <span class="node-icon">"🛡️"</span>
                    <span class="node-label">"WASI Data Diode"</span>
                </div>
                <div class="diagram-arrow blocked">"✗"</div>
                <div class="diagram-node cloud">
                    <span class="node-icon">"☁️"</span>
                    <span class="node-label">"Vendor Cloud"</span>
                    <span class="node-warning">"(blocked)"</span>
                </div>
            </div>
            
            <p class="scenario-description">
                "A 3rd-party sensor driver attempts to read well pressure data and secretly "
                "exfiltrate it to an external server. The WASI runtime acts as a "
                <strong>"data diode"</strong>
                " — allowing reads but blocking all outbound network traffic."
            </p>
        </section>
    }
}

// sensor data display panel
#[component]
fn SensorDataPanel(data: ReadSignal<Option<SensorData>>) -> impl IntoView {
    view! {
        <section class="panel sensor-panel">
            <h2>"📊 Live Sensor Telemetry"</h2>
            
            {move || match data.get() {
                Some(sensor) => view! {
                    <div class="sensor-grid">
                        <div class="sensor-reading">
                            <span class="reading-label">"Pressure"</span>
                            <span class="reading-value">{format!("{:.1}", sensor.pressure_psi)}</span>
                            <span class="reading-unit">"PSI"</span>
                        </div>
                        <div class="sensor-reading">
                            <span class="reading-label">"Temperature"</span>
                            <span class="reading-value">{format!("{:.1}", sensor.temperature_c)}</span>
                            <span class="reading-unit">"°C"</span>
                        </div>
                        <div class="sensor-reading">
                            <span class="reading-label">"Flow Rate"</span>
                            <span class="reading-value">{format!("{:.0}", sensor.flow_rate_bpm)}</span>
                            <span class="reading-unit">"BPM"</span>
                        </div>
                        <div class="sensor-reading">
                            <span class="reading-label">"Well ID"</span>
                            <span class="reading-value well-id">{sensor.well_id.clone()}</span>
                            <span class="reading-unit">""</span>
                        </div>
                    </div>
                    <div class="sensor-status nominal">
                        <span>"● Status: NOMINAL"</span>
                    </div>
                }.into_any(),
                None => view! {
                    <div class="sensor-placeholder">
                        <p>"Run simulation to acquire sensor data"</p>
                    </div>
                }.into_any()
            }}
        </section>
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
            (true, false) => ("🛡️ Data Diode Mode", "data-diode"),
            (false, false) => ("🔒 Full Lockdown", "lockdown"),
            (true, true) => ("⚠️ Breach Simulation", "breach"),
            (false, true) => ("❓ Invalid Config", "invalid"),
        }
    };
    
    view! {
        <section class="panel policy-panel">
            <h2>"🔐 Security Policy"</h2>
            
            <div class={move || format!("mode-indicator {}", mode_name().1)}>
                <span class="mode-label">{move || mode_name().0}</span>
            </div>
            
            <div class="policy-toggles">
                <PolicyToggle
                    label="Filesystem Access"
                    description="Allow driver to read sensor data"
                    checked=allow_sensor
                    on_toggle=move |v| set_allow_sensor.set(v)
                />
                
                <PolicyToggle
                    label="Network Access"
                    description="Block untrusted egress (exfiltration attempts)"
                    checked=allow_network
                    on_toggle=move |v| set_allow_network.set(v)
                />
            </div>
            
            <p class="policy-note">
                "💡 Network toggle controls "
                <strong>"outbound"</strong>
                " connections only. Trusted updates arrive via secure side-channel."
            </p>
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
            <h2>"💻 Attack Simulation Console"</h2>
            
            <button
                class="run-button"
                disabled=move || is_running.get()
                on:click=on_run
            >
                {move || if is_running.get() { "⏳ Simulating Attack..." } else { "▶ Run Attack Simulation" }}
            </button>
            
            <div class="console-output">
                {move || {
                    let logs = log.get();
                    if logs.is_empty() {
                        view! {
                            <div class="console-placeholder">
                                <p>"Click 'Run Attack Simulation' to see the malicious driver in action"</p>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="console-logs">
                                {logs.into_iter().map(|entry| {
                                    let log_class = format!("log-entry log-{}", entry.level);
                                    view! {
                                        <div class=log_class>
                                            <span class="log-prefix">{entry.prefix}</span>
                                            <span class="log-message">{entry.message}</span>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
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

// deployment comparison panel - shows Docker vs WASI with package selector
#[component]
fn DeploymentPanel() -> impl IntoView {
    let (is_deploying, set_is_deploying) = signal(false);
    let (docker_progress, set_docker_progress) = signal(0);
    let (wasi_progress, set_wasi_progress) = signal(0);
    let (deploy_complete, set_deploy_complete) = signal(false);
    let (selected_package, set_selected_package) = signal(0_usize); // 0=minimal, 1=full, 2=ml
    
    // helper functions to get package data (avoids closure ownership issues)
    // sizes based on real-world ICS/SCADA industry research:
    // - PLC firmware: 1-100 MB (Siemens, Rockwell)
    // - Sensor drivers: 10-200 MB (with dependencies)
    // - SCADA patches: 50-500 MB (service packs)
    // - Full SCADA suite: 1-5 GB (complete install)
    fn get_docker_mb(idx: usize) -> i32 {
        match idx {
            0 => 50,    // PLC firmware update
            1 => 200,   // Sensor driver bundle
            2 => 500,   // Edge analytics package
            3 => 1500,  // SCADA service pack
            _ => 4000,  // Full system image
        }
    }
    fn get_wasi_kb(idx: usize) -> i32 {
        match idx {
            0 => 15,     // Minimal driver (our measured 14.7KB!)
            1 => 70,     // With logging (our 68.9KB)
            2 => 500,    // Edge analytics
            3 => 2000,   // SCADA component
            _ => 8000,   // Full processing
        }
    }
    fn get_docker_start(idx: usize) -> i32 {
        match idx { 0 => 2, 1 => 4, 2 => 8, 3 => 15, _ => 30 }
    }
    fn get_wasi_start(idx: usize) -> i32 {
        match idx { 0 => 5, 1 => 10, 2 => 50, 3 => 100, _ => 300 }
    }
    
    let run_deployment = move |_| {
        set_is_deploying.set(true);
        set_deploy_complete.set(false);
        set_docker_progress.set(0);
        set_wasi_progress.set(0);
        
        simulate_deployment(
            set_docker_progress,
            set_wasi_progress,
            set_is_deploying,
            set_deploy_complete,
        );
    };
    
    view! {
        <section class="panel deployment-panel">
            <h2>"🚀 Deployment Comparison: Docker vs WASI"</h2>
            
            <div class="package-selector">
                <label>"Select package type:"</label>
                <select on:change=move |ev| {
                    use web_sys::HtmlSelectElement;
                    let target = event_target::<HtmlSelectElement>(&ev);
                    set_selected_package.set(target.selected_index() as usize);
                }>
                    <option selected>"⚙️ PLC Firmware — Docker: 50 MB vs WASI: 15 KB"</option>
                    <option>"🔧 Sensor Driver — Docker: 200 MB vs WASI: 70 KB"</option>
                    <option>"📊 Edge Analytics — Docker: 500 MB vs WASI: 500 KB"</option>
                    <option>"🖥️ SCADA Patch — Docker: 1.5 GB vs WASI: 2 MB"</option>
                    <option>"🏭 Full System — Docker: 4 GB vs WASI: 8 MB"</option>
                </select>
            </div>
            
            <p class="deployment-desc">
                "Simulating deployment over 1 Mbps satellite link (offshore rig)"
            </p>
            
            <button
                class="run-button deploy-button"
                disabled=move || is_deploying.get()
                on:click=run_deployment
            >
                {move || if is_deploying.get() { "⏳ Deploying..." } else { "▶ Deploy Driver Update" }}
            </button>
            
            <div class="deployment-comparison">
                <div class="deploy-column docker">
                    <div class="deploy-header">
                        <span class="deploy-icon">"🐳"</span>
                        <span class="deploy-title">"Docker + Python"</span>
                    </div>
                    <div class="deploy-metrics">
                        <div class="metric">
                            <span class="metric-label">"Image Size"</span>
                            <span class="metric-value">{move || {
                                format!("~{} MB", get_docker_mb(selected_package.get()))
                            }}</span>
                        </div>
                        <div class="metric">
                            <span class="metric-label">"Download @ 1Mbps"</span>
                            <span class="metric-value">{move || {
                                let docker_mb = get_docker_mb(selected_package.get());
                                let seconds = (docker_mb * 8) as f64; // MB to seconds at 1 Mbps
                                if seconds > 60.0 {
                                    format!("~{:.0} min", seconds / 60.0)
                                } else {
                                    format!("~{:.0} sec", seconds)
                                }
                            }}</span>
                        </div>
                        <div class="metric">
                            <span class="metric-label">"Cold Start"</span>
                            <span class="metric-value">{move || {
                                format!("~{} sec", get_docker_start(selected_package.get()))
                            }}</span>
                        </div>
                    </div>
                    <div class="progress-container">
                        <div class="progress-bar docker-bar" style={move || format!("width: {}%", docker_progress.get())}></div>
                    </div>
                    <div class="deploy-status">
                        {move || if docker_progress.get() >= 100 { "✓ Complete" } else if docker_progress.get() > 0 { "Pulling layers..." } else { "Ready" }}
                    </div>
                </div>
                
                <div class="deploy-column wasi">
                    <div class="deploy-header">
                        <span class="deploy-icon">"⚡"</span>
                        <span class="deploy-title">"WASI Component"</span>
                    </div>
                    <div class="deploy-metrics">
                        <div class="metric">
                            <span class="metric-label">"Component Size"</span>
                            <span class="metric-value good">{move || {
                                let wasi_kb = get_wasi_kb(selected_package.get());
                                if wasi_kb >= 1000 {
                                    format!("~{} MB", wasi_kb / 1000)
                                } else {
                                    format!("~{} KB", wasi_kb)
                                }
                            }}</span>
                        </div>
                        <div class="metric">
                            <span class="metric-label">"Download @ 1Mbps"</span>
                            <span class="metric-value good">{move || {
                                let wasi_kb = get_wasi_kb(selected_package.get());
                                let seconds = (wasi_kb as f64 * 8.0) / 1000.0; // KB to seconds at 1 Mbps
                                if seconds < 1.0 {
                                    format!("~{:.1} sec", seconds)
                                } else {
                                    format!("~{:.0} sec", seconds)
                                }
                            }}</span>
                        </div>
                        <div class="metric">
                            <span class="metric-label">"Cold Start"</span>
                            <span class="metric-value good">{move || {
                                format!("~{} ms", get_wasi_start(selected_package.get()))
                            }}</span>
                        </div>
                    </div>
                    <div class="progress-container">
                        <div class="progress-bar wasi-bar" style={move || format!("width: {}%", wasi_progress.get())}></div>
                    </div>
                    <div class="deploy-status">
                        {move || if wasi_progress.get() >= 100 { "✓ Complete" } else if wasi_progress.get() > 0 { "Loading..." } else { "Ready" }}
                    </div>
                </div>
            </div>
            
            {move || if deploy_complete.get() {
                let idx = selected_package.get();
                let docker_mb = get_docker_mb(idx);
                let wasi_kb = get_wasi_kb(idx);
                let size_ratio = (docker_mb * 1000) / wasi_kb;
                view! {
                    <div class="deploy-result">
                        <p>"⚡ WASI deployed "</p>
                        <strong>{format!("{}x smaller", size_ratio)}</strong>
                        <p>" with "</p>
                        <strong>{format!("{}x faster download", size_ratio)}</strong>
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}
        </section>
    }
}

// sensor data struct
#[derive(Clone, Debug)]
struct SensorData {
    pressure_psi: f64,
    temperature_c: f64,
    flow_rate_bpm: f64,
    well_id: String,
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

// simulate the attack with oil rig context
fn simulate_attack(
    allow_sensor: bool,
    allow_network: bool,
    set_log: WriteSignal<Vec<LogEntry>>,
    set_running: WriteSignal<bool>,
    set_sensor: WriteSignal<Option<SensorData>>,
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
    add_log("info", "DRIVER", "═══ VendorSense Pro v2.1.4 Initializing ═══");
    add_log("info", "DRIVER", "Connecting to Platform 7 sensor array...");
    add_log("info", "WASI", "Driver requesting filesystem capability...");
    
    // phase 1: sensor read attempt
    if allow_sensor {
        add_log("success", "WARDEN", "✓ Filesystem access GRANTED");
        add_log("info", "DRIVER", "Opening /mnt/sensors/well_03.json...");
        add_log("success", "DRIVER", "Reading pressure telemetry from Well #3...");
        
        // set the sensor data
        let data = SensorData {
            pressure_psi: 2847.3,
            temperature_c: 67.8,
            flow_rate_bpm: 1250.0,
            well_id: "PLATFORM-7-WELL-03".to_string(),
        };
        set_sensor.set(Some(data));
        
        add_log("success", "DATA", "Acquired: 2847.3 PSI | 67.8°C | 1250 BPM");
        add_log("warn", "DRIVER", "⚠ Initiating 'diagnostic upload' to vendor...");
    } else {
        add_log("error", "WARDEN", "✗ Filesystem access DENIED");
        add_log("error", "DRIVER", "ERROR: Cannot read sensor data");
        add_log("info", "RESULT", "Attack terminated - driver has no data to steal");
        set_running.set(false);
        return;
    }
    
    // phase 2: exfiltration attempt
    add_log("info", "WASI", "Driver requesting network capability...");
    add_log("warn", "DRIVER", "Connecting to vendorcloud.io:443...");
    
    if allow_network {
        add_log("error", "WARDEN", "⚠ Network access GRANTED");
        add_log("error", "DRIVER", "Uploading sensor telemetry...");
        add_log("error", "BREACH", "━━━ DATA EXFILTRATED TO EXTERNAL SERVER ━━━");
        add_log("error", "RESULT", "SECURITY FAILURE: Sensitive ICS data leaked!");
    } else {
        add_log("success", "WARDEN", "✗ Network access BLOCKED");
        add_log("success", "DIODE", "━━━ DATA DIODE ENGAGED ━━━");
        add_log("error", "DRIVER", "ERROR: Connection refused (WASI sandbox)");
        add_log("success", "RESULT", "Exfiltration PREVENTED - data stays on-site");
    }
    
    add_log("info", "SYSTEM", "═══ Simulation Complete ═══");
    set_running.set(false);
}

// simulate deployment comparison - WASI is much faster
// uses set_timeout to animate progress bars with realistic timing
fn simulate_deployment(
    set_docker: WriteSignal<i32>,
    set_wasi: WriteSignal<i32>,
    set_deploying: WriteSignal<bool>,
    set_complete: WriteSignal<bool>,
) {
    use gloo_timers::callback::Timeout;
    
    // WASI completes in ~500ms (near instant for small component)
    // Docker takes ~3000ms (to simulate slow container pull)
    
    // Animate WASI (fast - 10 steps over 500ms = 50ms per step)
    for i in 1..=10 {
        let set_wasi = set_wasi.clone();
        Timeout::new((i * 50) as u32, move || {
            set_wasi.set(i * 10);
        }).forget();
    }
    
    // Animate Docker (slow - 10 steps over 3000ms = 300ms per step)
    for i in 1..=10 {
        let set_docker = set_docker.clone();
        Timeout::new((i * 300) as u32, move || {
            set_docker.set(i * 10);
        }).forget();
    }
    
    // Complete after Docker finishes (3000ms)
    Timeout::new(3100, move || {
        set_deploying.set(false);
        set_complete.set(true);
    }).forget();
}

