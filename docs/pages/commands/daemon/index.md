# cadman daemon

Daemon control commands (FR-2).

- `daemon start` – start the Cadman background service
- `daemon stop` – stop the service
- `daemon reload` – reload configuration without downtime
- `daemon status` – show current state

> Notes
> - Linux: **systemd**; macOS: **launchd**.
> - Caddy reloads are validated and zero-downtime (NFR-10).
