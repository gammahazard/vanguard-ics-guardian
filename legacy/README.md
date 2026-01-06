# Legacy Comparison Dockerfiles

This directory contains Docker-based implementations of the same functionality
as our WASI components. These exist to provide a **verifiable baseline** for
size and performance comparisons.

> **Note:** These are intentionally "typical" implementations using common
> Python data science libraries. They are not optimized for size.

## Package Sizes

| Dockerfile | Description | Expected Size |
|------------|-------------|---------------|
| `minimal.Dockerfile` | Basic sensor driver | ~200 MB |
| `full.Dockerfile` | With data processing | ~800 MB |
| `ml.Dockerfile` | ML inference engine | ~2 GB |

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
