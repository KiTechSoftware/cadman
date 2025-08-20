# Component Design

## CLI Commands (by area)

- **Self‑management**: `install`, `update`, `uninstall`, `doctor`, `configure`, `info`, `logs`, `ports`
- **Daemon**: `daemon start|stop|reload|status`
- **Project**: `init|up|down|purge`
- **App**: `app add|remove|list (ls)|info|status|ports|logs`
- **Wrappers**: `pod|compose|caddy`

## Cross‑cutting modules

- **Config**: load/merge precedence (CLI > ENV > global config > defaults).
- **Registry**: read/write `apps.toml` with file locking; generate ULIDs.
- **Allocator**: manage ranges, ensure socket availability and registry uniqueness.
- **Logging**: file logs by default (`~/.cadman/logs` or `/var/log/cadman`).
- **Output**: `table|json|yaml` with JSON `schema_version`.
- **Caddy**: produce JSON or Caddyfile; validate and reload safely.
- **Daemon**: adapters for systemd (Linux) and launchd (macOS).
