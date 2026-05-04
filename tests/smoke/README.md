# Cadman v0.1.0 Smoke Tests

These smoke tests describe the manual verification path for the v0.1.0 MVP.

Run them on Linux with Podman and Caddy installed. The system-install and non-dry reconcile paths require a correctly configured `cadman` user/group, writable Cadman state paths, and rootless Podman subuid/subgid ranges where applicable.

## Smoke Set

| File | Purpose |
| --- | --- |
| `user_install.md` | Verifies the user install and project workflow. |
| `labeled_container_reconcile.md` | Verifies labeled Podman container discovery and Caddy route reconciliation. |
| `project_scan_reconcile.md` | Verifies recursive project discovery, registration, and reconcile. |
| `system_install.md` | Verifies system install planning and health checks. |

## Local Script

The local script runs non-destructive checks:

```sh
scripts/smoke-v0.1.0.sh
```

It builds Cadman, checks the version, lists config/registry paths, runs dry-run reconcile, doctor, and status.

Set `CADMAN_BIN=/path/to/cadman` to test an installed binary instead of `cargo run`.

Set `CADMAN_SMOKE_MUTATE=1` to also run `config init`, `init`, `add`, `registry list`, and `scan` in a temporary project. Use that only in a prepared user-scope environment or disposable system-scope environment.

## Known Limitations

Non-dry `cadman reconcile` depends on the host Podman and Caddy setup. If Podman cannot initialize rootless state, if the `cadman` user lacks subuid/subgid ranges, or if the `cadman` user cannot access the working directory, the command can fail for host setup reasons rather than Cadman workflow reasons.
