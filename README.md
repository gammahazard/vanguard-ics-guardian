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
</p>

---

## 🎯 The Scenario: Oil Rig Data Exfiltration

> *"A 3rd-party sensor driver attempts to read pressure data and secretly exfiltrate it to a vendor cloud. Our runtime acts as a Data Diode—allowing the read but physically blocking the network connection."*

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
| **Dashboard** | HTML/JS | Real-time security console |

## 🔧 Tech Stack

- **Standard:** WASI 0.2 (Preview 2) Component Model
- **Guest Language:** Rust with `cargo-component`
- **Host Runtime:** JavaScript via `@bytecodealliance/jco`
- **Interface Definition:** WIT with `wit-bindgen`

## 📁 Project Structure

```
vanguard-ics-guardian/
├── wit/                    # WASI interface definitions
│   └── world.wit
├── guest/                  # Rust WASM component (the "attacker")
│   ├── Cargo.toml
│   └── src/lib.rs
├── host/                   # JavaScript runtime (the "warden")
│   ├── shim/
│   │   ├── filesystem.js   # Mock wasi:filesystem
│   │   └── sockets.js      # Mock wasi:sockets (blocks connections)
│   └── runner.mjs
└── docs/
    └── BRANCHING.md        # Development workflow
```

## 🚀 Quick Start

```bash
# Build the malicious driver
cd guest && cargo component build --release

# Transpile and run
cd ../host && npm install && npm run build && npm run demo
```

## 📊 Security Scenarios

| Mode | Filesystem | Network | Description |
|------|------------|---------|-------------|
| 🛡️ **Data Diode** | ✓ Allow | ✗ Block | *Production mode* |
| 🔒 **Full Lockdown** | ✗ Block | ✗ Block | Zero trust |
| ⚠️ **Compromised** | ✓ Allow | ✓ Allow | Breach simulation |

## 🌿 Branch Strategy

This project uses feature branches to demonstrate professional Git workflow:

| Branch | Purpose |
|--------|---------|
| `main` | Stable releases |
| `develop` | Integration branch |
| `feature/wit-interface` | WIT definitions |
| `feature/rust-guest` | Malicious driver implementation |
| `feature/js-host` | Warden runtime shims |
| `feature/web-dashboard` | Security console UI |

## 📜 License

MIT © 2026

---

<p align="center">
  <em>Built to demonstrate capability-based security for industrial control systems.</em>
</p>
