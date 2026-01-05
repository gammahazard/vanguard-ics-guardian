# Branching Strategy

This project follows a **Git Flow**-inspired branching model to demonstrate professional version control practices.

## Branch Overview

```
main ─────────────────────────────────────────────────▶ (stable releases)
  │
  └──▶ develop ───────────────────────────────────────▶ (integration)
         │
         ├──▶ feature/wit-interface ──────┐
         │                                │
         ├──▶ feature/rust-guest ─────────┼──▶ merge to develop
         │                                │
         ├──▶ feature/js-host ────────────┤
         │                                │
         └──▶ feature/web-dashboard ──────┘
```

## Branch Purposes

| Branch | Base | Purpose | Merge Target |
|--------|------|---------|--------------|
| `main` | - | Production-ready code | - |
| `develop` | `main` | Integration of features | `main` |
| `feature/wit-interface` | `develop` | WIT interface definitions | `develop` |
| `feature/rust-guest` | `develop` | Rust WASM guest component | `develop` |
| `feature/js-host` | `develop` | JavaScript host shims | `develop` |
| `feature/web-dashboard` | `develop` | Security console UI | `develop` |

## Workflow

1. **Start feature**: `git checkout -b feature/xxx develop`
2. **Develop**: Make commits with conventional messages
3. **Pull Request**: Open PR to `develop`
4. **Review & Merge**: Squash or merge commit
5. **Release**: Merge `develop` → `main` with version tag

## Commit Convention

```
type(scope): description

feat(guest): implement sensor file reading
fix(host): correct error code mapping
docs(readme): add architecture diagram
```
