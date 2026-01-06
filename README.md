<p align="center">
  <img src="https://img.shields.io/badge/WASI-0.2%20Preview%202-blueviolet?style=for-the-badge&logo=webassembly" alt="WASI 0.2"/>
  <img src="https://img.shields.io/badge/Rust-1.75+-orange?style=for-the-badge&logo=rust" alt="Rust"/>
  <img src="https://img.shields.io/badge/Component%20Model-Compliant-success?style=for-the-badge" alt="Component Model"/>
</p>

<h1 align="center">🛡️ Vanguard ICS Guardian</h1>

<p align="center">
  <strong>A high-assurance security simulation demonstrating capability-based sandboxing<br/>using WASI 0.2, the Component Model, and a custom "Data Diode" runtime.</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/status-in%20development-yellow" alt="Status"/>
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="License"/>
  <img src="https://img.shields.io/badge/PRs-welcome-brightgreen" alt="PRs Welcome"/>
  <img src="https://img.shields.io/badge/mobile-responsive-blueviolet" alt="Mobile Responsive"/>
</p>

---

## 🎯 The Scenario: Oil Rig Data Exfiltration

> *"A 3rd-party sensor driver on an offshore oil rig attempts to read pressure data and secretly exfiltrate it to a vendor cloud. Our WASI runtime acts as a Data Diode—allowing the read but blocking all outbound network connections."*

```
┌─────────────────────────────────────────────────────────────┐
│                    VANGUARD ICS GUARDIAN                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ┌─────────────┐      wasi:filesystem      ┌───────────┐  │
│   │  Malicious  │ ──────────────────────────▶│  ✓ ALLOW  │  │
│   │   Driver    │                           └───────────┘  │
│   │   (WASM)    │      wasi:sockets/tcp     ┌───────────┐  │
│   │             │ ─────────────────────────▶│  ✗ BLOCK  │  │
│   └─────────────┘                           └───────────┘  │
│                                                             │
│   "Data Diode Mode: Read sensor → Block exfiltration"      │
└─────────────────────────────────────────────────────────────┘
```

## 🏗️ Architecture

| Component | Technology | Purpose |
|-----------|------------|---------|
| **Guest** | Rust → WASM | Untrusted sensor driver attempting data theft |
| **Host** | JavaScript (JCO) | The "Warden" runtime controlling capabilities |
| **Interface** | WIT (WASI 0.2) | Standard capability contracts |
| **Dashboard** | Rust + Leptos | Real-time security console (compiles to WASM) |

📖 **[Read full architecture doc →](docs/ARCHITECTURE.md)** - WASI vs Docker comparison

## 🔧 Tech Stack

- **Standard:** WASI 0.2 (Preview 2) Component Model
- **Guest Language:** Rust with `cargo-component`
- **Host Runtime:** JavaScript via `@bytecodealliance/jco`
- **Dashboard:** Leptos (Rust reactive web framework)
- **Interface Definition:** WIT with `wit-bindgen`

## 📁 Project Structure

```
vanguard-ics-guardian/
├── wit/                    # WASI interface definitions
│   └── world.wit
├── guest/                  # Rust WASM (the "attacker")
│   └── src/lib.rs
├── host/                   # JavaScript runtime (the "warden")
│   ├── shim/
│   │   ├── filesystem.js   # Mock wasi:filesystem
│   │   └── sockets.js      # Data diode + secure channel
│   └── test/
│       └── shims.test.js   # 18 unit tests
├── dashboard/              # Leptos web UI
│   ├── src/lib.rs
│   └── styles.css          # Mobile-responsive
└── docs/
    ├── ARCHITECTURE.md     # WASI vs Docker rationale
    └── BRANCHING.md        # Git workflow
```

## 🚀 Quick Start

**Run the Dashboard:**
```bash
# Install trunk (build tool for Leptos)
cargo install trunk

# Run dev server with live reload
cd dashboard && trunk serve
# Opens http://localhost:8080
```

**Run the Host Demo:**
```bash
cd host && npm install && npm run demo
```

## 📊 Security Modes

| Mode | Filesystem | External | Internal | Description |
|------|:----------:|:--------:|:--------:|-------------|
| 🛡️ **Data Diode** | ✓ Allow | ✗ Block | ✗ Block | *Production mode* |
| � **Secure Channel** | ✓ Allow | ✗ Block | ✓ Allow | Internal SCADA only |
| �🔒 **Full Lockdown** | ✗ Block | ✗ Block | ✗ Block | Zero trust |
| ⚠️ **Breach** | ✓ Allow | ✓ Allow | ✓ Allow | Security failure demo |

**Approved Internal Endpoints (Secure Channel mode):**
- `10.0.0.50:502` - SCADA server (Modbus)
- `10.0.0.51:102` - PLC gateway (S7)
- `192.168.100.10:443` - Data historian

## 🧪 Testing

```bash
# JavaScript host tests (18 tests)
cd host && npm test

# Rust guest tests
cd guest && cargo test
```

## 🏭 IEC 62443 Alignment

This project demonstrates key principles from the **IEC 62443** industrial cybersecurity standard:

| IEC 62443 Principle | Our Implementation |
|---------------------|-------------------|
| **Zone & Conduit Model** | OT zone (sensors) isolated from IT/Cloud via data diode |
| **Defense in Depth** | WASI capability model adds runtime-level security layer |
| **Least Privilege** | Components only receive explicitly granted capabilities |
| **Secure by Default** | Network access denied unless specifically allowed |
| **Unidirectional Gateways** | Data Diode mode: read IN, block OUT |

> ⚠️ **Note:** This is a demonstration/simulation, not a certified IEC 62443 product. Formal compliance requires third-party assessment.

## 📡 Bandwidth Reality: Remote Deployments

For offshore oil rigs with limited satellite connectivity (~1 Mbps):

| Package | Docker (~500 MB) | WASI (~200 KB) |
|---------|:----------------:|:--------------:|
| **Download Time** | ~67 minutes | ~1.6 seconds |
| **Network Impact** | Saturates link | Negligible |
| **Failover Speed** | Minutes | Milliseconds |

*This is why WASI matters for remote ICS environments.*

## 🌿 Branch Strategy

| Branch | Purpose | Deployment |
|--------|---------|------------|
| `main` | Stable releases | Production |
| `develop` | Integration | Preview |
| `feature/*` | Feature work | — |

## 📜 License

MIT © 2026

---

<p align="center">
  <em>Built to demonstrate capability-based security for industrial control systems.</em>
</p>

