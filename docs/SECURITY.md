# Security Analysis

## Attack Surface

### What We Protect Against

| Attack Vector | Traditional Approach | WASI Sandbox |
|--------------|---------------------|--------------|
| **Data Exfiltration** | Network egress possible | No network capability (Data Diode) |
| **Lateral Movement** | Pivot to other systems | No socket access unless granted |
| **Config Tampering** | Write to filesystem | Read-only filesystem capability |
| **Supply Chain Attack** | Malicious driver code | Sandboxed, capabilities deny-by-default |
| **Buffer Overflow** | Process crash or RCE | Sandbox trap, ~0.2ms rebuild |
| **Path Traversal** | Access sensitive files | Capability restricts to `/mnt/sensor_data.json` |

### The WASI Security Boundary

```
┌─────────────────────────────────────────────────────────────────┐
│                         UNTRUSTED                               │
│                   (3rd-Party Sensor Driver)                     │
│                                                                 │
│   ┌───────────────────────────────────────────────────────────┐ │
│   │               WASM LINEAR MEMORY                          │ │
│   │                                                           │ │
│   │   • 32-bit address space (max 4GB)                        │ │
│   │   • No access to host memory                              │ │
│   │   • No syscalls                                           │ │
│   │   • No file handles (unless granted)                      │ │
│   │   • No network sockets (unless granted)                   │ │
│   │   • Can only call imported functions                      │ │
│   └───────────────────────────────────────────────────────────┘ │
│                                                                 │
└────────────────────────────────────────┬────────────────────────┘
                                         │ WIT Interface
                                         │ (capability boundary)
┌────────────────────────────────────────▼────────────────────────┐
│                         TRUSTED                                 │
│                    (WASI Host Runtime)                          │
│                                                                 │
│   Host provides ONLY:                                           │
│   • wasi:filesystem (read /mnt/sensor_data.json)                │
│   • wasi:cli (stdout for logging)                               │
│                                                                 │
│   Host DENIES:                                                  │
│   • wasi:sockets (no network egress = DATA DIODE)               │
│   • Filesystem write access                                     │
│   • Process spawning                                            │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Security Modes

The Vanguard dashboard demonstrates 4 security configurations:

| Mode | Filesystem | External Net | Internal Net | Use Case |
|------|:----------:|:------------:|:------------:|----------|
| 🛡️ **Data Diode** | ✓ | ✗ | ✗ | Production - sensor read only |
| 🔗 **Secure Channel** | ✓ | ✗ | ✓ | Internal SCADA endpoints only |
| 🔒 **Full Lockdown** | ✗ | ✗ | ✗ | Zero trust, deny all |
| ⚠️ **Breach** | ✓ | ✓ | ✓ | Demo only - shows data exfiltration |

### Approved Internal Endpoints (Secure Channel)

When Secure Channel mode is enabled, only these endpoints are whitelisted:

```
10.0.0.50:502      SCADA server (Modbus)
10.0.0.51:102      PLC gateway (S7comm)
192.168.100.10:443 On-site data historian
```

All other network destinations are blocked, including external cloud services.

## Attack Scenario: Malicious Sensor Driver

The demo simulates a supply chain attack where a 3rd-party driver attempts to:

1. **Read sensor data** - Acquires well pressure/temperature readings
2. **Exfiltrate to cloud** - Attempts TCP connection to `vendorcloud.io:443`

### Data Diode Protection

```
┌──────────────┐         ┌──────────────┐         ┌──────────────┐
│   Pressure   │  read   │  3rd-Party   │  BLOCK  │   Vendor     │
│    Sensor    │ ───────►│    Driver    │ ───X───►│    Cloud     │
│              │   ✓     │   (WASM)     │         │   (exfil)    │
└──────────────┘         └──────────────┘         └──────────────┘
                                │
                                │ Capability check
                                ▼
                         wasi:sockets = DENIED
                         "connection-refused"
```

The driver successfully reads sensor data (legitimate function) but cannot exfiltrate because the WASI runtime refuses to grant network capabilities.

## IEC 62443 Alignment

### Zone and Conduit Model

```
┌─────────────────────────────────────────────────────────────────┐
│  LEVEL 0-2: OT ZONE (Plant)                                     │
│                                                                 │
│   ┌─────────┐     ┌─────────────────┐     ┌─────────────────┐  │
│   │ Sensors │────►│  WASI Gateway   │────►│ Internal SCADA  │  │
│   │ (L0)    │     │  (Conduit L1-2) │     │ (L2)            │  │
│   └─────────┘     └────────┬────────┘     └─────────────────┘  │
│                            │                                    │
│                            │ DATA DIODE                         │
│                            │ (blocks L4-5)                      │
│                            ▼                                    │
└─────────────────────────────────────────────────────────────────┘
                             X
                    ─────────┼─────────
                             │ BLOCKED
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│  LEVEL 4-5: IT ZONE (Enterprise/Cloud)                          │
│                                                                 │
│   ┌─────────────────┐                                          │
│   │  Vendor Cloud   │  ← Cannot receive exfiltrated data       │
│   │  (vendorcloud.io)│                                          │
│   └─────────────────┘                                          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Principle: Least Privilege

The WASM component has **zero** capabilities by default:
- ❌ Filesystem access
- ❌ Network access  
- ❌ Process spawning
- ❌ Environment variables

Capabilities are granted explicitly per-mode:
- **Data Diode**: ✅ Filesystem (read-only), ❌ Network
- **Secure Channel**: ✅ Filesystem, ✅ Approved internal IPs only
- **Full Lockdown**: ❌ All capabilities denied

### Principle: Defense in Depth

```
Layer 1: Rust type safety (compile time)
Layer 2: WASM sandbox (memory isolation)
Layer 3: WASI capability model (deny-by-default)
Layer 4: Host policy enforcement (Data Diode)
Layer 5: 2oo3 TMR for availability (crash recovery)
```

## 2oo3 Fault Tolerance

The dashboard demonstrates Triple Modular Redundancy for crash recovery:

| Metric | Python Multiprocessing | WASM Hot-Swap |
|--------|:----------------------:|:-------------:|
| Rebuild time | ~3 seconds | **~0.2ms (measured)** |
| Downtime | 2-5 seconds | **0ms** |
| Frames lost during fault | Many | **0** |
| Host crash rate | 0% | 0% |

### How 2oo3 Voting Works

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Instance 1    │    │   Instance 2    │    │   Instance 3    │
│   2847.3 PSI    │    │   2847.3 PSI    │    │    FAULTY       │
│   ● HEALTHY     │    │   ● HEALTHY     │    │   ✗ REBUILDING  │
└────────┬────────┘    └────────┬────────┘    └────────┬────────┘
         │                      │                      │
         └──────────────────────┼──────────────────────┘
                                │
                                ▼
                    ┌───────────────────────┐
                    │   MAJORITY VOTE       │
                    │   Result: 2847.3 PSI  │
                    │   ✓ Valid (2/3)       │
                    └───────────────────────┘
```

- **Zero switchover delay** - 2/3 instances still voting during rebuild
- **Async rebuild** - Faulty instance reinstantiates in ~0.2ms
- **Continuous operation** - Sensor readings never interrupted

This demonstrates SIL 2/3 voting patterns (IEC 61508) at the software layer. *Note: This is an architectural demonstration, not a certified safety system.*

## What Each Technology Solves

Understanding the **boundaries** of each technology is critical.

### Rust

| ✅ Solves | ❌ Doesn't Solve |
|-----------|-----------------|
| Buffer overflows | Logic bugs |
| Use-after-free | Algorithm errors |
| Data races | Business logic mistakes |
| Null pointer dereference | Incorrect sensor readings |

### WASM

| ✅ Solves | ❌ Doesn't Solve |
|-----------|-----------------|
| Memory isolation (sandbox) | Logic bugs |
| Trap instead of crash | Network-level security |
| No ambient syscall access | Encryption |
| Deterministic execution | Authentication |

### WASI

| ✅ Solves | ❌ Doesn't Solve |
|-----------|-----------------|
| Capability-based security | Network encryption |
| Deny-by-default permissions | User authentication |
| Explicit host control | Access control policies |
| No ambient authority | Audit logging |

### 2oo3 TMR

| ✅ Solves | ❌ Doesn't Solve |
|-----------|-----------------|
| Software fault recovery (~0ms) | Network path failure |
| Crash containment | Hardware failure |
| Zero data loss on trap | Power failure |
| Continuous availability | Byzantine faults |

## Complementary Technologies Still Needed

| Concern | WASM/WASI/Rust? | What You Need |
|---------|:---------------:|---------------|
| Memory safety | ✅ | — |
| Sandbox isolation | ✅ | — |
| Capability control | ✅ | — |
| Fault recovery | ✅ | — |
| Network encryption | ❌ | TLS/mTLS |
| Authentication | ❌ | Certificates, OAuth |
| Network redundancy | ❌ | PRP/HSR (IEC 62439-3) |
| Hardware redundancy | ❌ | Dual servers, failover |
| Logic correctness | ❌ | Unit tests, fuzzing |

## Recommended Hardening

Beyond WASI sandboxing, production deployments should consider:

1. **Cryptographic signing** — Verify WASM component hash before loading
2. **Resource limits** — Cap memory and CPU per WASM instance
3. **Audit logging** — Log all capability denials and crashes
4. **Rate limiting** — Detect rapid crash/restart patterns
5. **TLS everywhere** — Encrypt sensor data in transit
6. **mTLS** — Mutual authentication between gateway components
7. **Component attestation** — Verify WASM binary integrity at runtime

## Size Comparison: Attack Surface

| Metric | Docker + Python | WASI Component |
|--------|:---------------:|:--------------:|
| Binary size | ~50-200 MB | **14.7 KB** |
| Dependencies | 1000s of packages | **0 external deps** |
| Syscalls available | 300+ | **0** (host-controlled) |
| Network by default | ✓ Full access | **✗ Denied** |
| Filesystem by default | ✓ Full access | **✗ Denied** |

*Smaller attack surface = fewer vulnerabilities to exploit.*
