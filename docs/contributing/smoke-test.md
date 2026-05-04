# Smoke Test Guide

This document is intended for developers and CI operators who need to run the repository smoke tests.

## Purpose

The smoke script `scripts/smoke-v0.1.0.sh` exercises end-to-end flows in a controlled way. It supports a dry-run mode (no mutations) and an optional mutating mode for prepared test hosts.

## Location

- Script: `scripts/smoke-v0.1.0.sh`
- Scenario docs: `tests/smoke/README.md`

## Usage

### Dry-run (safe)

```bash
./scripts/smoke-v0.1.0.sh
```

### Allow mutation (ONLY on a prepared test host and after code review):

```bash
CADMAN_SMOKE_MUTATE=1 ./scripts/smoke-v0.1.0.sh
```

## Environment & prerequisites

- Linux host with Podman and Caddy installed.
- Ensure you have a safe test environment; the mutating mode will write generated Caddy site files and attempt a reload.

## Notes for CI

- Prefer running the smoke script in dry-run mode in CI to validate generation and planner output without mutating system state.
- If integrating into an integration pipeline, use an isolated VM with Podman and Caddy configured for test safety.
