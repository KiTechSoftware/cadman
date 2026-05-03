# Project Scan Reconcile Smoke

## Goal

Verify recursive project discovery, registration, and project-config route reconciliation.

## Steps

```sh
root="$(mktemp -d)"
mkdir -p "$root/apps/web"
cd "$root/apps/web"
cadman init
cd "$root"
cadman scan "$root"
cadman scan --add "$root"
cadman registry list
cadman --dry-run reconcile
cadman reconcile
cadman status
```

## Expected Result

```txt
scan finds cadman.toml recursively
scan --add registers the discovered project
registry list shows a project-config app
dry-run reconcile plans project-config routes
reconcile writes deterministic generated Caddy site files
status shows route and site state after reconcile
```

## Notes

The default project config publishes loopback ports. Ensure those ports are available, or edit `cadman.toml` before running non-dry reconcile.
