# Node.js CLI Demo

Proves the same WASM component runs **outside the browser** - demonstrating "browser → edge" portability.

## Quick Start

```bash
cd cli
node run.mjs
```

## What It Does

1. **Loads** the compiled WASM component (same binary as browser demo)
2. **Measures** real instantiation time (~0.1-1ms)
3. **Displays** security policy enforcement

## Expected Output

```
╔══════════════════════════════════════════════════════════╗
║       VANGUARD ICS GUARDIAN - Node.js CLI Demo           ║
╚══════════════════════════════════════════════════════════╝

Loading WASM component...
  ✓ Loaded: 14.7 KB
  ✓ Load time: 0.85ms

Compiling WASM module...
  ✓ Compile time: 2.34ms

Benchmarking instantiation (2oo3 rebuild simulation)...
  ✓ Iterations: 10
  ✓ Average: 0.152ms
  ✓ Min: 0.098ms
  ✓ Max: 0.234ms

Security Policy Demonstration:

  [Data Diode (Production)]
    Filesystem: ✓ ALLOW
    Network:    ✗ BLOCK

  [Secure Channel]
    Filesystem: ✓ ALLOW
    Network:    ⚠ INTERNAL ONLY
...
```

## Why This Matters

| Environment | Instantiation | Same Binary? |
|-------------|:-------------:|:------------:|
| Browser (Leptos) | ~0.1ms | ✓ |
| Node.js (this demo) | ~0.1ms | ✓ |
| Wasmtime (edge) | ~0.1ms | ✓ |

The same `.wasm` file runs everywhere - proving WASI's portability claim.
