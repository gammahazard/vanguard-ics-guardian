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
         ├──▶ feature/rust-guest ─────────┼
         │                                │
         ├──▶ feature/js-host ────────────┼──▶ merge to develop
         │                                │
         ├──▶ feature/web-dashboard ──────┤
         │                                │
         └──▶ feature/secure-channel ─────┘
```

## Branch Purposes

| Branch | Base | Purpose | Merge Target |
|--------|------|---------|--------------||
| `main` | - | Production-ready code | - |
| `develop` | `main` | Integration of features | `main` |
| `feature/wit-interface` | `develop` | WIT interface definitions | `develop` |
| `feature/rust-guest` | `develop` | Rust WASM guest component | `develop` |
| `feature/js-host` | `develop` | JavaScript host shims | `develop` |
| `feature/web-dashboard` | `develop` | Security console UI | `develop` |
| `feature/secure-channel` | `develop` | Approved endpoints whitelist | `develop` |
| `feature/polyfill-legacy` | `develop` | Polyfill WIT + Docker comparison | `develop` |

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
