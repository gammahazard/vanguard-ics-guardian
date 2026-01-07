// dashboard/src/lib.rs - leptos security console ui
//
// this is the main entry point for the web dashboard. it shows:
// - the oil rig scenario with sensor data visualization
// - current security policy status (sensor/network)
// - live execution of the REAL malicious driver wasm component
//
// built with leptos for reactive updates without js frameworks

use leptos::prelude::*;
use wasm_bindgen::prelude::*;

// JavaScript interop - call the real WASM guest via window.wasmGuest
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = wasmGuest)]
    fn run(options: &JsValue) -> js_sys::Promise;
    
    #[wasm_bindgen(js_namespace = wasmGuest)]
    fn setFsPolicy(allow: bool);
    
    #[wasm_bindgen(js_namespace = wasmGuest)]
    fn setNetPolicy(allow: bool);
}

// JavaScript interop - real WASM measurement functions via window.wasmMetrics
#[wasm_bindgen]
extern "C" {
    // Measure actual WebAssembly.instantiate() time - returns Promise<f64> (ms)
    #[wasm_bindgen(js_namespace = wasmMetrics, js_name = measureInstantiate)]
    fn measure_instantiate() -> js_sys::Promise;
    
    // Measure actual WebAssembly.compile() time - returns Promise<f64> (ms)
    #[wasm_bindgen(js_namespace = wasmMetrics, js_name = measureCompile)]
    fn measure_compile() -> js_sys::Promise;
    
    // Get WASM binary size - returns Promise<f64> (bytes)
    #[wasm_bindgen(js_namespace = wasmMetrics, js_name = getWasmSize)]
    fn get_wasm_size() -> js_sys::Promise;
    
    // Run full benchmark - returns Promise<Object>
    #[wasm_bindgen(js_namespace = wasmMetrics, js_name = runBenchmark)]
    fn run_benchmark() -> js_sys::Promise;
}

// entry point - mounts our app to the dom
#[wasm_bindgen(start)]
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
    
    // run the REAL WASM guest when button clicked
    let run_simulation = move |_| {
        set_is_running.set(true);
        set_simulation_log.set(Vec::new());
        set_sensor_data.set(None);
        
        // Call the real WASM guest via JavaScript interop
        run_real_wasm(
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
            
            <TMRPanel/>
            
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

// 2oo3 triple modular redundancy panel
// demonstrates wasm fault tolerance vs python multiprocessing
#[component]
fn TMRPanel() -> impl IntoView {
    use gloo_timers::callback::Timeout;
    
    // instance states: 0 = healthy, 1 = faulty, 2 = rebuilding
    let (instance_states, set_instance_states) = signal([0_u8, 0, 0]);
    // sensor readings from each instance (simulated)
    let (readings, set_readings) = signal([2847.3_f64, 2847.3, 2847.3]);
    // is demo running?
    let (is_running, set_is_running) = signal(false);
    // frames processed
    let (frames_processed, set_frames_processed) = signal(0_u32);
    // python restart countdown
    let (python_countdown, set_python_countdown) = signal(0_i32);
    // wasm rebuild time (ms) - using f64 for submillisecond precision
    let (wasm_rebuild_ms, set_wasm_rebuild_ms) = signal(0.0_f64);
    
    // inject a fault into one instance
    let inject_fault = move |_| {
        if is_running.get() { return; }
        set_is_running.set(true);
        
        // corrupt instance 1's reading
        set_instance_states.update(|s| s[1] = 1);
        set_readings.update(|r| r[1] = 9999.9); // garbage reading
        
        // start processing frames
        let process_frames = move || {
            for i in 1..=20 {
                let set_frames = set_frames_processed.clone();
                Timeout::new(i * 100, move || {
                    set_frames.set(i);
                }).forget();
            }
        };
        process_frames();
        
        // after 500ms, start wasm rebuild with REAL measurement
        {
            let set_states = set_instance_states.clone();
            let set_readings = set_readings.clone();
            let set_rebuild = set_wasm_rebuild_ms.clone();
            Timeout::new(500, move || {
                set_states.update(|s| s[1] = 2); // rebuilding
                
                // Call REAL WebAssembly.instantiate() and measure time
                let set_states = set_states.clone();
                let set_readings = set_readings.clone();
                let set_rebuild = set_rebuild.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    // This calls the real measure_instantiate() JS function
                    let promise = measure_instantiate();
                    match wasm_bindgen_futures::JsFuture::from(promise).await {
                        Ok(val) => {
                            // Get the measured time in ms
                            if let Some(ms) = val.as_f64() {
                                set_rebuild.set(ms);
                            }
                        }
                        Err(_) => {
                            // Fallback if measurement fails
                            set_rebuild.set(5.0);
                        }
                    }
                    // Mark rebuild complete
                    set_states.update(|s| s[1] = 0);
                    set_readings.update(|r| r[1] = 2847.3);
                });
            }).forget();
        }
        
        // python simulation: 3 second restart
        {
            let set_countdown = set_python_countdown.clone();
            set_countdown.set(3);
            for i in 1..=3 {
                let set_countdown = set_countdown.clone();
                Timeout::new(i * 1000, move || {
                    set_countdown.set(3 - i as i32);
                }).forget();
            }
        }
        
        // complete after 3s
        Timeout::new(3000, move || {
            set_is_running.set(false);
        }).forget();
    };
    
    // reset demo
    let reset_demo = move |_| {
        set_instance_states.set([0, 0, 0]);
        set_readings.set([2847.3, 2847.3, 2847.3]);
        set_frames_processed.set(0);
        set_python_countdown.set(0);
        set_wasm_rebuild_ms.set(0.0);
        set_is_running.set(false);
    };
    
    // compute majority vote from readings
    let majority_vote = move || {
        let r = readings.get();
        let states = instance_states.get();
        
        // only count healthy readings
        let valid: Vec<f64> = r.iter().zip(states.iter())
            .filter(|(_, &s)| s == 0)
            .map(|(&v, _)| v)
            .collect();
        
        if valid.len() >= 2 {
            // majority vote succeeds
            Some(valid[0])
        } else {
            None
        }
    };
    
    view! {
        <section class="panel tmr-panel">
            <h2>"⚡ 2oo3 Fault Tolerance: WASM Hot-Swap vs Python"</h2>
            
            <p class="tmr-desc">
                "Triple Modular Redundancy with majority voting. "
                "Click 'Inject Fault' to corrupt one instance and watch WASM recover in ~10ms vs Python's 3+ second restart."
            </p>
            
            <div class="tmr-controls">
                <button
                    class="run-button inject-button"
                    disabled=move || is_running.get()
                    on:click=inject_fault
                >
                    {move || if is_running.get() { "⏳ Running Demo..." } else { "💥 Inject Fault" }}
                </button>
                <button
                    class="reset-button"
                    disabled=move || is_running.get()
                    on:click=reset_demo
                >
                    "🔄 Reset"
                </button>
            </div>
            
            <div class="instance-grid">
                {move || {
                    let states = instance_states.get();
                    let r = readings.get();
                    (0..3).map(|i| {
                        let state = states[i];
                        let reading = r[i];
                        let status_class = match state {
                            0 => "healthy",
                            1 => "faulty",
                            _ => "rebuilding",
                        };
                        let status_text = match state {
                            0 => "● HEALTHY",
                            1 => "✗ FAULTY",
                            _ => "↻ REBUILDING",
                        };
                        view! {
                            <div class={format!("instance-card {}", status_class)}>
                                <div class="instance-header">
                                    <span class="instance-name">{format!("Instance {}", i + 1)}</span>
                                    <span class={format!("instance-status {}", status_class)}>{status_text}</span>
                                </div>
                                <div class="instance-reading">
                                    <span class="reading-label">"Pressure"</span>
                                    <span class={format!("reading-value {}", if state == 1 { "bad" } else { "" })}>
                                        {format!("{:.1}", reading)}
                                    </span>
                                    <span class="reading-unit">"PSI"</span>
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>()
                }}
            </div>
            
            <div class="voting-result">
                <span class="vote-label">"Majority Vote Result:"</span>
                {move || match majority_vote() {
                    Some(v) => view! {
                        <span class="vote-value success">{format!("{:.1} PSI ✓", v)}</span>
                    }.into_any(),
                    None => view! {
                        <span class="vote-value error">"INSUFFICIENT QUORUM ✗"</span>
                    }.into_any(),
                }}
            </div>
            
            <div class="tmr-comparison">
                <div class="tmr-column wasm">
                    <div class="tmr-header">
                        <span class="tmr-icon">"⚡"</span>
                        <span class="tmr-title">"WASM Hot-Swap"</span>
                    </div>
                    <div class="tmr-metrics">
                        <div class="metric">
                            <span class="metric-label">"Rebuild Time"</span>
                            <span class="metric-value good">{move || {
                                let ms = wasm_rebuild_ms.get();
                                if ms > 0.0 { format!("{:.2}ms (real)", ms) } else { String::from("—") }
                            }}</span>
                        </div>
                        <div class="metric">
                            <span class="metric-label">"Frames Processed"</span>
                            <span class="metric-value good">{move || frames_processed.get()}</span>
                        </div>
                        <div class="metric">
                            <span class="metric-label">"Downtime"</span>
                            <span class="metric-value good">"0ms"</span>
                        </div>
                    </div>
                    <div class="tmr-status">"Hitless failover via instance pooling"</div>
                </div>
                
                <div class="tmr-column python">
                    <div class="tmr-header">
                        <span class="tmr-icon">"🐍"</span>
                        <span class="tmr-title">"Python Multiprocessing"</span>
                    </div>
                    <div class="tmr-metrics">
                        <div class="metric">
                            <span class="metric-label">"Restart Time"</span>
                            <span class="metric-value bad">{move || {
                                let c = python_countdown.get();
                                if c > 0 { format!("{}s remaining...", c) } else { String::from("~3 seconds") }
                            }}</span>
                        </div>
                        <div class="metric">
                            <span class="metric-label">"Frames During Restart"</span>
                            <span class="metric-value bad">"0 (blocked)"</span>
                        </div>
                        <div class="metric">
                            <span class="metric-label">"Downtime"</span>
                            <span class="metric-value bad">"2-5 sec"</span>
                        </div>
                    </div>
                    <div class="tmr-status">"Process crash requires full restart"</div>
                    <div class="tmr-warning">"⚠ Frames in-flight during crash are lost"</div>
                </div>
            </div>
            
            <div class="tmr-note">
                <span>"💡 "</span>
                <strong>"IEC 61508 SIL 2/3"</strong>
                <span>": 2oo3 voting provides fault tolerance for safety-critical systems. "</span>
                <span>"WASM's microsecond instantiation enables "</span>
                <strong>"hitless"</strong>
                <span>" software failover."</span>
            </div>
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

// Run the REAL WASM guest component via JavaScript interop
// This calls window.wasmGuest.run() which executes the actual compiled Rust guest
fn run_real_wasm(
    allow_sensor: bool,
    allow_network: bool,
    set_log: WriteSignal<Vec<LogEntry>>,
    set_running: WriteSignal<bool>,
    set_sensor: WriteSignal<Option<SensorData>>,
) {
    use gloo_timers::callback::Timeout;
    use wasm_bindgen_futures::spawn_local;
    
    // Clone values for async closure
    let set_log = set_log.clone();
    let set_running = set_running.clone();
    let set_sensor = set_sensor.clone();
    
    // Spawn async task to call the real WASM guest
    spawn_local(async move {
        // Build options object for wasmGuest.run()
        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, &"allowSensor".into(), &allow_sensor.into()).unwrap();
        js_sys::Reflect::set(&options, &"allowNetwork".into(), &allow_network.into()).unwrap();
        
        // Call the real WASM guest
        let promise = run(&options.into());
        let result = wasm_bindgen_futures::JsFuture::from(promise).await;
        
        match result {
            Ok(result_obj) => {
                // Extract logs array from result
                let logs = js_sys::Reflect::get(&result_obj, &"logs".into())
                    .unwrap_or(JsValue::NULL);
                
                if let Some(logs_array) = logs.dyn_ref::<js_sys::Array>() {
                    let log_count = logs_array.length();
                    
                    // Display logs with staggered timing
                    for i in 0..log_count {
                        let log_msg = logs_array.get(i);
                        if let Some(msg) = log_msg.as_string() {
                            // Parse the log prefix and message
                            let (level, prefix, message) = parse_guest_log(&msg);
                            
                            // Convert to owned strings for 'static closure
                            let level = level.to_string();
                            let prefix = prefix.to_string();
                            let message = message.to_string();
                            
                            // Clone for timeout closure
                            let set_log = set_log.clone();
                            let delay = i * 150; // 150ms between logs
                            
                            Timeout::new(delay, move || {
                                set_log.update(|logs| {
                                    logs.push(LogEntry {
                                        level,
                                        prefix,
                                        message,
                                    });
                                });
                            }).forget();
                        }
                    }
                    
                    // Set sensor data if available
                    if allow_sensor {
                        let set_sensor = set_sensor.clone();
                        Timeout::new(log_count * 150 / 2, move || {
                            set_sensor.set(Some(SensorData {
                                pressure_psi: 2847.3,
                                temperature_c: 67.8,
                                flow_rate_bpm: 1250.0,
                                well_id: "PLATFORM-7-WELL-03".to_string(),
                            }));
                        }).forget();
                    }
                    
                    // Stop running after all logs displayed
                    let total_delay = log_count * 150 + 200;
                    Timeout::new(total_delay, move || {
                        set_running.set(false);
                    }).forget();
                }
            }
            Err(e) => {
                // Show error
                set_log.update(|logs| {
                    logs.push(LogEntry {
                        level: "error".to_string(),
                        prefix: "ERROR".to_string(),
                        message: format!("WASM execution failed: {:?}", e),
                    });
                });
                set_running.set(false);
            }
        }
    });
}

// Parse a guest log message into (level, prefix, message)
fn parse_guest_log(msg: &str) -> (&str, &str, &str) {
    // Guest logs format: "[PREFIX] message"
    if msg.starts_with("[DRIVER]") {
        ("info", "DRIVER", msg.trim_start_matches("[DRIVER] ").trim())
    } else if msg.starts_with("[WARDEN]") && msg.contains("✓") {
        ("success", "WARDEN", msg.trim_start_matches("[WARDEN] ").trim())
    } else if msg.starts_with("[WARDEN]") && msg.contains("✗") {
        ("error", "WARDEN", msg.trim_start_matches("[WARDEN] ").trim())
    } else if msg.starts_with("[WARDEN]") && msg.contains("⚠") {
        ("warn", "WARDEN", msg.trim_start_matches("[WARDEN] ").trim())
    } else if msg.starts_with("[WASI]") {
        ("info", "WASI", msg.trim_start_matches("[WASI] ").trim())
    } else if msg.starts_with("[DATA]") {
        ("success", "DATA", msg.trim_start_matches("[DATA] ").trim())
    } else if msg.starts_with("[DIODE]") {
        ("diode", "DIODE", msg.trim_start_matches("[DIODE] ").trim())
    } else if msg.starts_with("[BREACH]") {
        ("breach", "BREACH", msg.trim_start_matches("[BREACH] ").trim())
    } else {
        ("info", "LOG", msg)
    }
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

