# Commands

Per SRS §5.1.

## Global flags

- `--verbose` `--debug` `--quiet`
- Read-only commands support: `--output table|json|yaml`

## Categories

- **Self-management**: `install`, `update`, `uninstall`, `doctor`, `configure`, `info`, `logs`, `ports`
- **Daemon**: `daemon start|stop|reload|status`
- **Project**: `init`, `up`, `down`, `purge`
- **App**: `app add|remove|list (ls)|info|status|ports|logs`
- **Wrappers**: `pod`, `compose`, `caddy`

**App selectors** (where relevant): `--id | --name | --path` (priority: id > name > path).
