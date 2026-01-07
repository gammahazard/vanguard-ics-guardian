# Legacy Comparison: Docker + Python

This directory contains Docker-based implementations of the same functionality
as our WASI components. These exist to provide a **verifiable baseline** for
size and performance comparisons.

> **Note:** These are production-style implementations using common Python
> industrial libraries. They demonstrate the bloat that WASI eliminates.

## Python Driver Files

| File | Description | Dependencies |
|------|-------------|--------------|
| `sensor_minimal.py` | Basic Modbus sensor driver | pyserial, pymodbus |
| `sensor_full.py` | Data processing suite | pandas, numpy, scipy |
| `sensor_ml.py` | ML inference engine | tensorflow, onnxruntime, scikit-learn |

### Industry Patterns Demonstrated

All drivers implement proper industrial software patterns:

- **Structured logging** with ISO 8601 timestamps
- **Dataclasses** for immutable sensor readings
- **Error handling** with typed exceptions
- **Configuration management** via JSON files
- **ISA-95 naming conventions** for process variables
- **IEC 62443 security patterns** (documented attack vectors)

### The Security Story

Each driver includes a **malicious exfiltration attempt** that would:
- ✅ **Succeed** in a traditional Python/Docker environment
- ❌ **Fail** in our WASI sandbox (capability denied)

Look for the `_exfiltrate_to_cloud()` or `_send_telemetry()` methods.

## Package Sizes

| Dockerfile | Description | Expected Size | WASI Equivalent |
|------------|-------------|:-------------:|:---------------:|
| `minimal.Dockerfile` | Basic Modbus driver | **~50-200 MB** | 15-70 KB |
| `full.Dockerfile` | pandas/numpy processing | **~500-800 MB** | 500 KB |
| `ml.Dockerfile` | TensorFlow/ONNX inference | **~1.5-4 GB** | 2-8 MB |

> See the **dashboard dropdown** for the full comparison with download times at 1 Mbps satellite link.

## Building & Running

```bash
# Build all images
docker build -f minimal.Dockerfile -t legacy-minimal .
docker build -f full.Dockerfile -t legacy-full .
docker build -f ml.Dockerfile -t legacy-ml .

# Run the drivers
docker run --rm legacy-minimal
docker run --rm legacy-full
docker run --rm legacy-ml

# Check image sizes
docker images | grep legacy
```

## Inspecting Size

```bash
# Human-readable size
docker inspect legacy-minimal --format='{{.Size}}' | numfmt --to=iec

# Compare all
for img in minimal full ml; do
  echo "legacy-$img: $(docker inspect legacy-$img --format='{{.Size}}' | numfmt --to=iec)"
done
```

## Why This Matters

These serve as the **"villain"** in our security demo—showing what the
traditional approach looks like vs. WASI's capability-based sandboxing.

| Metric | Docker + Python | WASI Component |
|--------|:---------------:|:--------------:|
| **Minimal driver** | ~50 MB | **15 KB** (3,333x smaller) |
| **Cold start** | 2-5 seconds | **<0.1ms** |
| **Network access** | Unrestricted | Capability-denied |
| **Attack surface** | Full OS | Sandboxed |

## Real-World Context

These Python patterns are representative of actual ICS/SCADA drivers:

- **Modbus communication** - pymodbus for RTU/TCP
- **Data processing** - pandas/numpy for analytics
- **ML inference** - TensorFlow Lite for predictive maintenance
- **OPC UA integration** - asyncua for historian writes

The bloat comes from the Python ecosystem, not the code itself.
Our WASI components do the same job in 3,000x less space.
