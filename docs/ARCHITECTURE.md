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
│  │  WASM Component (~100KB)              │  │
│  │  └── Sensor driver (compiled Rust)    │  │
│  └───────────────────────────────────────┘  │
│                                             │
│  Capabilities granted at instantiation:     │
│  ✓ wasi:filesystem (read /mnt/sensors)     │
│  ✗ wasi:sockets (not granted)              │
└─────────────────────────────────────────────┘
Component Size: 50-500KB
Startup Time: <10ms (with compiled WASM)
Attack Surface: Only granted capabilities
```

## Key Differences

| Aspect | Docker + Python | WASI Component Model |
|--------|-----------------|---------------------|
| **Isolation Model** | Process/namespace | Capability-based |
| **Security Default** | Allow (must block) | Deny (must grant) |
| **Binary Size** | 500MB - 2GB | 50 - 500KB |
| **Cold Start** | 2-10 seconds | <10ms |
| **Memory Overhead** | 50-200MB | 1-10MB |
| **Static Analysis** | Limited | Full (linear memory model) |
| **Cross-Platform** | Linux-centric | True portability |

## How the Data Diode Works

### Capability Model

The WASI Component Model enforces security through **imports**. A component can only use capabilities it explicitly imports, and the host must provide them:

```wit
// world.wit - what the driver can access
world ics-guardian {
    import wasi:filesystem/types;     // ✓ granted
    import wasi:filesystem/preopens;  // ✓ granted
    import wasi:sockets/tcp;          // ✗ blocked by host
}
```

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
2. **Minimal TCB**: The Trusted Computing Base is tiny (~100KB runtime vs GB container)
3. **Auditable**: WIT interface definitions clearly show what code can access
4. **Fast Recovery**: <10ms startup means quick failover
5. **Edge-Ready**: Low memory/CPU overhead suits embedded deployments

## Future Vision

WASI and the Component Model are still maturing, but represent the direction for secure, portable computing:

- **WASI 0.2** (stable as of Jan 2024): Filesystem, sockets, clocks
- **WASI 0.3** (upcoming): Async, threading, more capabilities
- **Component Model**: Composable security boundaries

This project demonstrates these concepts are production-viable today.

---

*Built with Rust, WebAssembly, and the WASI 0.2 Component Model*
