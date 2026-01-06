# Architecture: WASI vs Docker for ICS Security

## Overview

This document explains why WebAssembly System Interface (WASI) and the Component Model represent a compelling alternative to Docker/Python for industrial control system (ICS) security—particularly for capability-based sandboxing of untrusted code.

## The Problem

Industrial control systems need to run third-party sensor drivers and data processing code. This code:
- Comes from vendors with varying security practices
- Needs access to sensitive operational data
- Must be prevented from exfiltrating that data
- Runs on resource-constrained edge devices

## Current Approach: Docker + Python

```
┌─────────────────────────────────────────────┐
│           Docker Container                   │
│  ┌───────────────────────────────────────┐  │
│  │  Python Runtime (~200MB)               │  │
│  │  ├── NumPy, Pandas, etc.              │  │
│  │  ├── OS libraries                      │  │
│  │  └── Vendor driver code               │  │
│  └───────────────────────────────────────┘  │
│  Full Linux namespace isolation             │
│  Network access: iptables rules             │
└─────────────────────────────────────────────┘
Container Size: 500MB - 2GB
Startup Time: 2-10 seconds
Attack Surface: Massive (entire Linux userland)
```

> **Verifiable:** See `legacy/` folder for example Dockerfiles that build to these sizes.
> - `minimal.Dockerfile` → ~200 MB (pyserial + pymodbus)
> - `full.Dockerfile` → ~800 MB (pandas, numpy, scipy)
> - `ml.Dockerfile` → ~2 GB (tensorflow, onnx)

**Limitations:**
- Containers isolate at the namespace level, not the capability level
- A compromised container can still access network unless explicitly blocked
- iptables rules are error-prone and external to the code
- Python's dynamic nature makes static analysis nearly impossible
- Cold start times matter for real-time industrial applications

## WASI Approach: Capability-Based Security

```
┌─────────────────────────────────────────────┐
│           WASI Runtime (Warden)              │
│  ┌───────────────────────────────────────┐  │
│  │  WASM Component (14.7KB actual)       │  │
│  │  └── Sensor driver (compiled Rust)    │  │
│  └───────────────────────────────────────┘  │
│                                             │
│  Capabilities granted at instantiation:     │
│  ✓ sensor-fs (read /mnt/sensors)           │
│  ✗ sensor-net (not granted)                │
└─────────────────────────────────────────────┘
Component Size: 14.7KB (measured)
Startup Time: <10ms
Attack Surface: Only granted capabilities
```

## Key Differences

| Aspect | Docker + Python | WASI Component Model |
|--------|-----------------|---------------------|
| **Isolation Model** | Process/namespace | Capability-based |
| **Security Default** | Allow (must block) | Deny (must grant) |
| **Binary Size** | 500MB - 2GB | **14.7KB** (measured) |
| **Cold Start** | 2-10 seconds | <10ms |
| **Memory Overhead** | 50-200MB | 1-10MB |
| **Static Analysis** | Limited | Full (linear memory model) |
| **Cross-Platform** | Linux-centric | True portability |

## How the Data Diode Works

### Capability Model

The WASI Component Model enforces security through **imports**. A component can only use capabilities it explicitly imports, and the host must provide them:

```wit
// world.wit - Standard WASI 0.2 Interfaces
package vanguard:ics;

world ics-guardian {
    // Restrict the standard filesystem interface
    import wasi:filesystem/types@0.2.0; 
    import wasi:filesystem/preopens@0.2.0;
    
    // Restrict the standard socket interface
    import wasi:sockets/tcp@0.2.0;
    
    export run: func();
}
```

> **Implementation Note:** Our demo uses a "polyfill" approach with locally-defined 
> interfaces (`sensor-fs`, `sensor-net`) that mirror the standard WASI behavior.
> This avoids registry dependency issues while demonstrating the same 
> capability-based security model that production WASI 0.2 implements.

### Host-Side Enforcement

The JavaScript host (Warden) implements these interfaces:

```javascript
// sockets.js - our data diode implementation
startConnect(network, remoteAddress) {
    if (!policy.allowNetwork) {
        // This is the data diode!
        return { tag: 'err', val: 'connection-refused' };
    }
    // ...
}
```

The driver code **cannot bypass this**. It has no other way to access the network because:
1. WASM has no syscalls—all I/O goes through imported functions
2. The host controls what implementations those imports receive
3. There's no escape hatch (no `/dev/`, no raw memory access)

## Security Modes

### Mode 1: Data Diode (Default)
- **Filesystem**: ✓ Allow reads from `/mnt/sensors`
- **Network**: ✗ Block all outbound connections
- **Use Case**: Production—data flows in, never out

### Mode 2: Secure Channel
- **Filesystem**: ✓ Allow sensor reads
- **Network**: ✓ Allow only to approved internal endpoints
- **Use Case**: Send data to internal SCADA, block external

### Mode 3: Full Lockdown
- **Filesystem**: ✗ Block all access
- **Network**: ✗ Block all connections
- **Use Case**: Zero-trust testing

## Enterprise Workflow

```
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│   feature   │ ──▶  │   develop   │ ──▶  │    main     │
│   branches  │      │  (preview)  │      │ (production)│
└─────────────┘      └─────────────┘      └─────────────┘
                           │                     │
                           ▼                     ▼
                     Preview Deploy        Production Deploy
                     (staging URL)         (production URL)
```

**Standard Workflow:**
1. Develop features on `feature/*` branches
2. Merge to `develop` → triggers preview deployment
3. Test on staging/preview environment
4. If everything works, merge `develop` → `main`
5. Production deployment triggered

## Why This Matters for ICS

1. **Defense in Depth**: Capabilities are enforced at the runtime level, not just network rules
2. **Minimal TCB**: The Trusted Computing Base is tiny (14.7KB component vs GB container)
3. **Auditable**: WIT interface definitions clearly show what code can access
4. **Fast Recovery**: <10ms startup means quick failover
5. **Edge-Ready**: Low memory/CPU overhead suits embedded deployments

## Future Vision

WASI and the Component Model are still maturing, but represent the direction for secure, portable computing:

- **WASI 0.2** (stable as of Jan 2024): Filesystem, sockets, clocks
- **WASI 0.3** (upcoming): Async, threading, more capabilities
- **Component Model**: Composable security boundaries

This project demonstrates these concepts are production-viable today.

## Secure Supply Chain Workflow (The "Air Gap" Model)

Unlike web apps, ICS updates must be cryptographically signed and verified before reaching the edge. This protects against **SolarWinds-style** supply chain attacks.

```
┌───────────────┐      ┌─────────────────┐      ┌──────────────────┐
│ Vendor Build  │ ───▶ │  Vanguard Hub   │ ───▶ │   Edge Device    │
│ (Rust Source) │      │ (Signing Auth)  │      │ (WASI Runtime)   │
└───────────────┘      └─────────────────┘      └──────────────────┘
       │                        │                        │
       ▼                        ▼                        ▼
  Compiles to            Adds Cryptographic        Verifies Sig &
  .wasm (15KB)           Signature (Ed25519)       Loads Component
```

### Why Ed25519?

| Algorithm | Key Size | Signature Size | Speed | Status |
|-----------|----------|----------------|-------|--------|
| RSA-2048 | 256 bytes | 256 bytes | Slow | Legacy |
| ECDSA P-256 | 64 bytes | 64 bytes | Medium | Common |
| **Ed25519** | **32 bytes** | **64 bytes** | **Fast** | **Modern** |

Ed25519 is ideal for resource-constrained edge devices: small, fast, and quantum-resistant designs are in progress.

### The Vanguard Hub Concept

The Hub acts as a **trust boundary** between vendor code and production systems:

1. **Vendor submits** compiled `.wasm` component
2. **Hub validates** against known-good hashes and scans for suspicious patterns
3. **Hub signs** with organization's Ed25519 private key
4. **Edge verifies** signature before loading—rejects unsigned/tampered code

> **Note:** This is a conceptual architecture. The signing infrastructure would be a future enhancement.

---

*Built with Rust, WebAssembly, and the WASI 0.2 Component Model*
