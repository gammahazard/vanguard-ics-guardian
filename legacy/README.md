# Legacy Comparison Dockerfiles

This directory contains Docker-based implementations of the same functionality
as our WASI components. These exist to provide a **verifiable baseline** for
size and performance comparisons.

> **Note:** These are intentionally "typical" implementations using common
> Python data science libraries. They are not optimized for size.

## Package Sizes

| Dockerfile | Description | Expected Size | WASI Equivalent |
|------------|-------------|---------------|-----------------|
| `minimal.Dockerfile` | Basic sensor driver | ~50-200 MB | **15-70 KB** |
| `full.Dockerfile` | With data processing | ~500-800 MB | **500 KB** |
| `ml.Dockerfile` | ML inference engine | ~1.5-4 GB | **2-8 MB** |

> See the **dashboard dropdown** for the full comparison with download times at 1 Mbps satellite link.

## Building

```bash
docker build -f minimal.Dockerfile -t legacy-minimal .
docker build -f full.Dockerfile -t legacy-full .
docker build -f ml.Dockerfile -t legacy-ml .
```

## Inspecting Size

```bash
docker inspect legacy-minimal --format='{{.Size}}'
```

## Why This Matters

These serve as the "villain" in our security demo—showing what the
traditional approach looks like vs. WASI's capability-based sandboxing.

**The ratio is dramatic:** A 50 MB Docker image vs. a 15 KB WASI component = **3,333x smaller**.

