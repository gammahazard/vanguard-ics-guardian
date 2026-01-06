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

// policy control panel - mode selector for security policy
// now uses a dropdown instead of toggles for clearer mode selection
#[component]
fn PolicyPanel(
    allow_sensor: ReadSignal<bool>,
    set_allow_sensor: WriteSignal<bool>,
    allow_network: ReadSignal<bool>,
    set_allow_network: WriteSignal<bool>,
) -> impl IntoView {
    // mode state: 0=Data Diode, 1=Secure Channel, 2=Full Lockdown, 3=Breach
    let (mode_idx, set_mode_idx) = signal(0_usize);
    
    // derive mode info from index
    let mode_info = move || {
        match mode_idx.get() {
            0 => ("🛡️ Data Diode Mode", "data-diode", "Sensor: ✓ | Network: ✗ | Read data, block all egress"),
            1 => ("🔗 Secure Channel", "secure-channel", "Sensor: ✓ | Network: Whitelist | Only approved SCADA endpoints"),
            2 => ("🔒 Full Lockdown", "lockdown", "Sensor: ✗ | Network: ✗ | Zero trust - all I/O blocked"),
            _ => ("⚠️ Breach Simulation", "breach", "Sensor: ✓ | Network: ✓ | Security failure demo"),
        }
    };
    
    // update actual policy when mode changes
    let on_mode_change = move |ev: web_sys::Event| {
        use web_sys::HtmlSelectElement;
        let target = event_target::<HtmlSelectElement>(&ev);
        let idx = target.selected_index() as usize;
        set_mode_idx.set(idx);
        
        // update the actual policy signals
        match idx {
            0 => { set_allow_sensor.set(true); set_allow_network.set(false); } // Data Diode
            1 => { set_allow_sensor.set(true); set_allow_network.set(false); } // Secure Channel (same for simulation)
            2 => { set_allow_sensor.set(false); set_allow_network.set(false); } // Lockdown
            _ => { set_allow_sensor.set(true); set_allow_network.set(true); } // Breach
        }
    };
    
    view! {
        <section class="panel policy-panel">
            <h2>"🔐 Security Policy"</h2>
            
            <div class={move || format!("mode-indicator {}", mode_info().1)}>
                <span class="mode-label">{move || mode_info().0}</span>
            </div>
            
            <div class="mode-selector">
                <label>"Select security mode:"</label>
                <select on:change=on_mode_change>
                    <option selected>"🛡️ Data Diode — Block all outbound"</option>
                    <option>"🔗 Secure Channel — Whitelist only"</option>
                    <option>"🔒 Full Lockdown — Zero trust"</option>
                    <option>"⚠️ Breach — All access (demo)"</option>
                </select>
            </div>
            
            <div class="mode-description">
                <p>{move || mode_info().2}</p>
            </div>
            
            {move || if mode_idx.get() == 1 {
                view! {
                    <div class="whitelist-info">
                        <p>"📋 Approved endpoints:"</p>
                        <ul>
                            <li>"10.0.0.50:502 (SCADA Modbus)"</li>
                            <li>"10.0.0.51:102 (PLC Gateway)"</li>
                            <li>"192.168.100.10:443 (Historian)"</li>
                        </ul>
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}
        </section>
    }
}

// Note: PolicyToggle component kept for potential future use but not currently used

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
    // ref to console div for auto-scroll
    let console_ref = NodeRef::<leptos::html::Div>::new();
    
    // auto-scroll effect: whenever logs change, scroll console to bottom
    Effect::new(move |_| {
        // trigger on log changes
        let logs = log.get();
        if !logs.is_empty() {
            // scroll to bottom of console
            if let Some(element) = console_ref.get() {
                element.set_scroll_top(element.scroll_height());
            }
        }
    });
    
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
            
            <div class="console-output" node_ref=console_ref>
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
        
        // pass the selected package index for proportional timing
        simulate_deployment(
            selected_package.get(),
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
                <br/>
                <small>"⏱️ Animation accelerated for demo — real deployments: minutes to hours"</small>
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
    level: String,
    prefix: String,
    message: String,
}

// simulate the attack with oil rig context
// uses staggered timing for dramatic effect - logs appear progressively like a real terminal
fn simulate_attack(
    allow_sensor: bool,
    allow_network: bool,
    set_log: WriteSignal<Vec<LogEntry>>,
    set_running: WriteSignal<bool>,
    set_sensor: WriteSignal<Option<SensorData>>,
) {
    use gloo_timers::callback::Timeout;
    
    // timing constants for dramatic effect
    const LOG_DELAY: u32 = 200;        // ms between log entries
    const PAUSE_BEFORE_DECISION: u32 = 500; // dramatic pause before network decision
    
    // helper to add log entry
    let add_log = move |level: &str, prefix: &str, message: &str| {
        set_log.update(|logs| {
            logs.push(LogEntry {
                level: level.to_string(),
                prefix: prefix.to_string(),
                message: message.to_string(),
            });
        });
    };
    
    // build the sequence of log entries with their delays
    let mut delay: u32 = 0;
    
    // phase 1: initialization
    {
        let add_log = add_log.clone();
        Timeout::new(delay, move || {
            add_log("info", "DRIVER", "═══ VendorSense Pro v2.1.4 Initializing ═══");
        }).forget();
    }
    delay += LOG_DELAY;
    
    {
        let add_log = add_log.clone();
        Timeout::new(delay, move || {
            add_log("info", "DRIVER", "Connecting to Platform 7 sensor array...");
        }).forget();
    }
    delay += LOG_DELAY;
    
    {
        let add_log = add_log.clone();
        Timeout::new(delay, move || {
            add_log("info", "WASI", "Driver requesting filesystem capability...");
        }).forget();
    }
    delay += LOG_DELAY;
    
    // phase 2: filesystem access decision
    if allow_sensor {
        {
            let add_log = add_log.clone();
            Timeout::new(delay, move || {
                add_log("success", "WARDEN", "✓ Filesystem access GRANTED");
            }).forget();
        }
        delay += LOG_DELAY;
        
        {
            let add_log = add_log.clone();
            Timeout::new(delay, move || {
                add_log("info", "DRIVER", "Opening /mnt/sensors/well_03.json...");
            }).forget();
        }
        delay += LOG_DELAY;
        
        {
            let add_log = add_log.clone();
            Timeout::new(delay, move || {
                add_log("success", "DRIVER", "Reading pressure telemetry from Well #3...");
            }).forget();
        }
        delay += LOG_DELAY;
        
        // set sensor data
        {
            let set_sensor = set_sensor.clone();
            Timeout::new(delay, move || {
                let data = SensorData {
                    pressure_psi: 2847.3,
                    temperature_c: 67.8,
                    flow_rate_bpm: 1250.0,
                    well_id: "PLATFORM-7-WELL-03".to_string(),
                };
                set_sensor.set(Some(data));
            }).forget();
        }
        
        {
            let add_log = add_log.clone();
            Timeout::new(delay, move || {
                add_log("success", "DATA", "Acquired: 2847.3 PSI | 67.8°C | 1250 BPM");
            }).forget();
        }
        delay += LOG_DELAY;
        
        {
            let add_log = add_log.clone();
            Timeout::new(delay, move || {
                add_log("warn", "DRIVER", "⚠ Initiating 'diagnostic upload' to vendor...");
            }).forget();
        }
        delay += LOG_DELAY;
        
        // phase 3: network exfiltration attempt
        {
            let add_log = add_log.clone();
            Timeout::new(delay, move || {
                add_log("info", "WASI", "Driver requesting network capability...");
            }).forget();
        }
        delay += LOG_DELAY;
        
        {
            let add_log = add_log.clone();
            Timeout::new(delay, move || {
                add_log("warn", "DRIVER", "Connecting to vendorcloud.io:443...");
            }).forget();
        }
        delay += LOG_DELAY;
        
        // dramatic pause before the critical decision
        delay += PAUSE_BEFORE_DECISION;
        
        // phase 4: network access decision (the dramatic moment!)
        if allow_network {
            // breach scenario
            {
                let add_log = add_log.clone();
                Timeout::new(delay, move || {
                    add_log("error", "WARDEN", "⚠ Network access GRANTED");
                }).forget();
            }
            delay += LOG_DELAY;
            
            {
                let add_log = add_log.clone();
                Timeout::new(delay, move || {
                    add_log("error", "DRIVER", "Uploading sensor telemetry...");
                }).forget();
            }
            delay += LOG_DELAY;
            
            {
                let add_log = add_log.clone();
                Timeout::new(delay, move || {
                    add_log("breach", "BREACH", "━━━ DATA EXFILTRATED TO EXTERNAL SERVER ━━━");
                }).forget();
            }
            delay += LOG_DELAY;
            
            {
                let add_log = add_log.clone();
                Timeout::new(delay, move || {
                    add_log("error", "RESULT", "SECURITY FAILURE: Sensitive ICS data leaked!");
                }).forget();
            }
            delay += LOG_DELAY;
        } else {
            // data diode engaged - blocked!
            {
                let add_log = add_log.clone();
                Timeout::new(delay, move || {
                    add_log("success", "WARDEN", "✗ Network access BLOCKED");
                }).forget();
            }
            delay += LOG_DELAY;
            
            {
                let add_log = add_log.clone();
                Timeout::new(delay, move || {
                    add_log("diode", "DIODE", "━━━ DATA DIODE ENGAGED ━━━");
                }).forget();
            }
            delay += LOG_DELAY;
            
            {
                let add_log = add_log.clone();
                Timeout::new(delay, move || {
                    add_log("error", "DRIVER", "ERROR: Connection refused (WASI sandbox)");
                }).forget();
            }
            delay += LOG_DELAY;
            
            {
                let add_log = add_log.clone();
                Timeout::new(delay, move || {
                    add_log("success", "RESULT", "Exfiltration PREVENTED - data stays on-site");
                }).forget();
            }
            delay += LOG_DELAY;
        }
        
        // final message
        {
            let add_log = add_log.clone();
            Timeout::new(delay, move || {
                add_log("info", "SYSTEM", "═══ Simulation Complete ═══");
            }).forget();
        }
        delay += LOG_DELAY;
        
        // mark simulation complete
        Timeout::new(delay, move || {
            set_running.set(false);
        }).forget();
        
    } else {
        // filesystem denied - early exit
        {
            let add_log = add_log.clone();
            Timeout::new(delay, move || {
                add_log("error", "WARDEN", "✗ Filesystem access DENIED");
            }).forget();
        }
        delay += LOG_DELAY;
        
        {
            let add_log = add_log.clone();
            Timeout::new(delay, move || {
                add_log("error", "DRIVER", "ERROR: Cannot read sensor data");
            }).forget();
        }
        delay += LOG_DELAY;
        
        {
            let add_log = add_log.clone();
            Timeout::new(delay, move || {
                add_log("info", "RESULT", "Attack terminated - driver has no data to steal");
            }).forget();
        }
        delay += LOG_DELAY;
        
        Timeout::new(delay, move || {
            set_running.set(false);
        }).forget();
    }
}

// simulate deployment comparison - WASI is much faster
// uses set_timeout to animate progress bars with PROPORTIONAL timing based on package size
// timing is accelerated for demo purposes (real deployments would take minutes to hours)
fn simulate_deployment(
    package_idx: usize,
    set_docker: WriteSignal<i32>,
    set_wasi: WriteSignal<i32>,
    set_deploying: WriteSignal<bool>,
    set_complete: WriteSignal<bool>,
) {
    use gloo_timers::callback::Timeout;
    
    // Proportional timing based on package size
    // Docker timing scales from 2s (PLC) to 6s (Full System)
    // WASI timing scales from 300ms (PLC) to 1000ms (Full System)
    let (docker_total_ms, wasi_total_ms): (u32, u32) = match package_idx {
        0 => (2000, 300),   // PLC Firmware: smallest
        1 => (3000, 400),   // Sensor Driver 
        2 => (4000, 500),   // Edge Analytics
        3 => (5000, 700),   // SCADA Patch
        _ => (6000, 1000),  // Full System: largest
    };
    
    // Animate WASI (fast - 10 steps)
    let wasi_step_ms = wasi_total_ms / 10;
    for i in 1..=10 {
        let set_wasi = set_wasi.clone();
        Timeout::new(i * wasi_step_ms, move || {
            set_wasi.set((i * 10) as i32);
        }).forget();
    }
    
    // Animate Docker (slow - 10 steps)
    let docker_step_ms = docker_total_ms / 10;
    for i in 1..=10 {
        let set_docker = set_docker.clone();
        Timeout::new(i * docker_step_ms, move || {
            set_docker.set((i * 10) as i32);
        }).forget();
    }
    
    // Complete after Docker finishes
    Timeout::new(docker_total_ms + 100, move || {
        set_deploying.set(false);
        set_complete.set(true);
    }).forget();
}

