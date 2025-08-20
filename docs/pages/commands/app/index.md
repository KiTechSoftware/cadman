# cadman app

Subcommands manage the application registry and runtime views.

- `app add` (FR-A1)
- `app remove [--purge]` (FR-A2)
- `app list` / `app ls` (FR-A3)
- `app info` (FR-A4)
- `app status` (FR-A5)
- `app ports` (FR-A6)
- `app logs` (FR-A7)

Selectors: `--id | --name | --path` with priority `id > name > path`.
