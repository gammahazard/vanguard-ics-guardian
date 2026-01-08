# Hardware Setup Guides

## Project 1: ICS Guardian (Pi 4 + Sensors)

**Data Flow:**
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      ICS GUARDIAN - HARDWARE DEMO                           │
│                                                                             │
│  SENSORS (Input)                      RASPBERRY PI 4                        │
│  ───────────────                      ──────────────                        │
│  ┌─────────────┐                   ┌────────────────────────────────────┐   │
│  │   DHT22     │  GPIO4            │          Rust Host (wasmtime)      │   │
│  │ Temp/Humid  │──────────────────►│                                    │   │
│  └─────────────┘                   │     ┌────────────────────────┐     │   │
│                                    │     │     guest.wasm         │     │   │
│  ┌─────────────┐                   │     │      (14.7 KB)         │     │   │
│  │   BME280    │  I2C (SDA/SCL)    │     │                        │     │   │
│  │ Temp/Press  │──────────────────►│     │  ✓ sensor-fs (read)    │     │   │
│  │ Humidity    │                   │     │  ✓ sensor-out (write)  │     │   │
│  └─────────────┘                   │     │  ✗ sensor-net BLOCKED  │◄────┼── BLOCKED!
│                                    │     │                        │     │   │
│  ┌─────────────┐                   │     └───────────┬────────────┘     │   │
│  │  DS18B20    │  1-Wire (GPIO17)  │                 │                  │   │
│  │ Waterproof  │──────────────────►│                 ▼                  │   │
│  └─────────────┘                   │     ┌────────────────────────┐     │   │
│                                    │     │   OUTPUT CONTROLLER    │     │   │
│                                    └─────┼────────────────────────┼─────┘   │
│                                          │                        │         │
│  OUTPUT (Physical)                       ▼                        ▼         │
│  ─────────────────    ┌─────────────────────────────────────────────────┐   │
│                       │                                                 │   │
│  ┌───────────────┐    │  ┌─────────────┐  ┌─────────────┐  ┌─────────┐ │   │
│  │ SainSmart     │◄───┼──│ RGB OLED    │  │ LED Strip   │  │ Buzzer  │ │   │
│  │ 4-CH Relay    │    │  │ (SPI)       │  │ (WS2812B)   │  │ (GPIO)  │ │   │
│  │ (GPIO 5,6,    │    │  │ Live temps  │  │ Status LEDs │  │ Alerts  │ │   │
│  │  13,19)       │    │  └─────────────┘  └─────────────┘  └─────────┘ │   │
│  │               │    │                                                 │   │
│  │  💡 CLICK!    │    └─────────────────────────────────────────────────┘   │
│  └───────────────┘                                                          │
│                                                                             │
│  Demo: Sensor reads → WASM processes → Relay clicks → Network BLOCKED      │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Wiring:**
```
Pi 4 GPIO                    INPUTS (Sensors)
─────────                    ────────────────
3.3V (Pin 1)  ────────────►  DHT22 VCC, BME280 VCC
GND (Pin 6)   ────────────►  DHT22 GND, BME280 GND, DS18B20 GND
GPIO4 (Pin 7) ────────────►  DHT22 DATA (+ 10kΩ pull-up)
SDA (Pin 3)   ────────────►  BME280 SDA
SCL (Pin 5)   ────────────►  BME280 SCL
GPIO17        ────────────►  DS18B20 DATA (+ 4.7kΩ pull-up)

Pi 4 GPIO                    OUTPUTS (SainSmart Relay)
─────────                    ─────────────────────────
5V (Pin 2)    ────────────►  Relay VCC
GND (Pin 14)  ────────────►  Relay GND
GPIO5         ────────────►  Relay IN1 (Channel 1)
GPIO6         ────────────►  Relay IN2 (Channel 2)
GPIO13        ────────────►  Relay IN3 (Channel 3)
GPIO19        ────────────►  Relay IN4 (Channel 4)
```

**Software:**
```bash
# Install wasmtime
curl https://wasmtime.dev/install.sh -sSf | bash

# Rust host with rppal for GPIO/I2C
cargo new pi-host && cd pi-host
cargo add wasmtime rppal dht-sensor bme280 ds18b20
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
