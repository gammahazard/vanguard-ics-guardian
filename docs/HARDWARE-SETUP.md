# Hardware Setup Guide

## Project 1: ICS Guardian (Pi 4 + Sensors)

**Data Flow:**
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      ICS GUARDIAN - HARDWARE DEMO                           │
│                                                                             │
│  SENSORS (Input)                      RASPBERRY PI 4 (guardian-node-1)     │
│  ───────────────                      ───────────────────────────────       │
│  ┌─────────────┐                   ┌────────────────────────────────────┐   │
│  │   BME280    │  I2C (SDA/SCL)    │          Rust Host (wasmtime)      │   │
│  │ Temp/Press  │──────────────────►│                                    │   │
│  │ Humidity    │                   │     ┌────────────────────────┐     │   │
│  │ @ 0x76      │                   │     │     guest.wasm         │     │   │
│  └─────────────┘                   │     │      (14.7 KB)         │     │   │
│                                    │     │                        │     │   │
│                                    │     │  ✓ i2c-sensor (read)   │     │   │
│                                    │     │  ✓ historian (log)     │────►│───► QNAP NAS
│                                    │     │  ✗ direct-net BLOCKED  │◄────┼── BLOCKED!
│                                    │     │                        │     │   │ (192.168.40.5)
│                                    │     └───────────┬────────────┘     │   │
│                                    │                 │                  │   │
│                                    │                 ▼                  │   │
│  OUTPUT (Physical)                 │     ┌────────────────────────┐     │   │
│  ─────────────────                 │     │   OUTPUT CONTROLLER    │     │   │
│                                    └─────┼────────────────────────┼─────┘   │
│                                          │                        │         │
│  ┌───────────────┐                       ▼                        ▼         │
│  │ SainSmart     │              ┌─────────────┐  ┌─────────────┐           │
│  │ Relay Module  │◄─────────────│ Status LEDs │  │ Buzzer      │           │
│  │ (Industrial)  │              │ (R/G/Y/B)   │  │ (Alerts)    │           │
│  └───────────────┘              └─────────────┘  └─────────────┘           │
│                                                                             │
│  Demo: BME280 read → WASM processes → TLS to QNAP → Relay actuation        │
└─────────────────────────────────────────────────────────────────────────────┘
```

> **Network:** Pi 4 at 192.168.40.4, QNAP NAS at 192.168.40.5 (Industrial Zone, segmented via UniFi Switch Lite 8 PoE)


**Wiring:**
```
Pi 4 GPIO                    INPUTS (BME280 Sensor)
─────────                    ──────────────────────
3.3V (Pin 1)  ────────────►  BME280 VCC
GND (Pin 6)   ────────────►  BME280 GND
SDA (Pin 3)   ────────────►  BME280 SDA
SCL (Pin 5)   ────────────►  BME280 SCL

Pi 4 GPIO                    OUTPUTS (SainSmart Relay)
─────────                    ─────────────────────────
5V (Pin 2)    ────────────►  Relay VCC
GND (Pin 14)  ────────────►  Relay GND
GPIO5         ────────────►  Relay IN1 (Industrial Fan)
GPIO6         ────────────►  Relay IN2 (Reserved)
```

**Software:**
```bash
# Install wasmtime
curl https://wasmtime.dev/install.sh -sSf | bash

# Rust host with rppal for GPIO/I2C
cargo new guardian-host && cd guardian-host
cargo add wasmtime rppal bme280
```

**What We Build:**
- Rust host that implements `sensor-fs` interface using `rppal`
- Reads real sensor data instead of mock JSON
- Same `guest.wasm` from browser demo (14.7 KB, no changes)
- Data diode enforced: network calls blocked at host level

**Implementation Guide:**

```
vanguard-ics-guardian/
├── guest/                      # ← NO CHANGES NEEDED
│   └── target/
│       └── guest.wasm          # Copy this to Pi (14.7 KB)
│
├── pi-host/                    # ← NEW: Create this folder
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs             # Wasmtime loader + sensor loop
│       ├── shim/
│       │   ├── mod.rs
│       │   ├── filesystem.rs   # Real GPIO/I2C reads
│       │   └── sockets.rs      # Block all (data diode)
│       └── display.rs          # RGB OLED output
```

| File | What It Does |
|------|--------------|
| `pi-host/src/main.rs` | Load `guest.wasm`, create shim imports, run loop |
| `pi-host/src/shim/filesystem.rs` | Replace mock JSON with real DHT22/BME280 reads |
| `pi-host/src/shim/sockets.rs` | Return `Err("blocked")` for all network calls |
| `pi-host/src/display.rs` | Write status to RGB OLED via SPI |

**Key Code (filesystem.rs):**
```rust
use rppal::{gpio::Gpio, i2c::I2c};
use dht_sensor::{dht22, DhtReading};

pub fn read_sensors() -> String {
    // DHT22 on GPIO4
    let gpio = Gpio::new().unwrap();
    let pin = gpio.get(4).unwrap().into_io(Mode::Output);
    let reading = dht22::read(&mut delay, &mut pin).unwrap();
    
    // BME280 on I2C
    let i2c = I2c::new().unwrap();
    let mut bme = Bme280::new_primary(i2c);
    let measurements = bme.measure().unwrap();
    
    format!(r#"{{"temp": {}, "humidity": {}, "pressure": {}}}"#,
        reading.temperature, reading.humidity, measurements.pressure)
}
```
