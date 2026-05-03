# Changelog

## 0.1.0

Initial MVP release.

### Added

- Linux-only, Podman-first, Caddy-first Cadman CLI.
- Project initialization with `cadman init`.
- Global config handling with discovery settings.
- Recursive project scanning with `cadman scan` and `cadman scan --add`.
- Registry commands for project-config and Podman-label sourced apps.
- Podman container visibility policy for current, cadman, and root scopes.
- Reconcile workflow that collects desired routes, registers explicit labeled containers, writes generated Caddy site files, validates Caddy, reloads only after successful validation, and updates state.
- Status and doctor reports with structured JSON output.
- Self lifecycle commands for install, update, uninstall, and healthcheck.
- Controlled wrapper commands for Caddy, Podman, and Podman Compose.
- CLI contract and smoke-test documentation.

### Not Included

- Docker runtime or Docker socket support.
- Traefik integration.
- Kubernetes support.
- macOS or Windows support.
- Public API or Web UI surfaces.
- Required daemon/live watcher runtime.
