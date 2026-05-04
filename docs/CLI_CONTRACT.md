# Cadman v0.1.0 CLI Contract

This document freezes the Cadman v0.1.0 command surface.

Cadman is Linux-only, Podman-first, and Caddy-first. Docker, Traefik, Kubernetes, macOS, Windows, the public API surface, the public Web UI surface, and a required daemon/live watcher runtime are out of scope for v0.1.0.

## Global Flags

Global flags apply to all subcommands:

| Flag | Behavior |
| --- | --- |
| `-v`, `--verbose` | Increase logging verbosity. Can be repeated. |
| `-q`, `--quiet` | Decrease logging verbosity. Can be repeated. |
| `--json` | Wrap output in the JSON envelope. |
| `--format <auto|text|markdown|json|jsonl|plain>` | Select payload rendering format. |
| `-p`, `--plain` | Output plain text. Conflicts with `--format`. |
| `--dry-run` | Plan actions without mutating files or services where supported. |
| `--color <auto|always|never>` | Select color output policy. |
| `-C`, `--cwd <PATH>` | Run against another working directory. |
| `--ci` | Non-interactive CI mode, assume yes. |
| `--non-interactive` | Non-interactive mode, assume no. |
| `-y`, `--yes` | Accept defaults automatically. |
| `--force` | Permit operations that normally skip or refuse duplicates/overwrites. |
| `--config <PATH>` | Use a specific Cadman global config path. |
| `--mode <current|cadman|root>` | Request a Cadman run mode. Effective mode still follows policy. |

## Output Contract

Human output is rendered by core through Scriba and `ctx.ui()`.

`--json` returns a JSON envelope with:

| Field | Meaning |
| --- | --- |
| `ok` | Whether the command completed successfully. |
| `format` | Payload format used inside the envelope. |
| `content` | Rendered structured content blocks. |

Commands that expose structured reports include a JSON block containing the full report payload. Text tables are presentation aids and are not the stable machine interface.

## Exit Codes

Cadman maps typed engine errors to stable classes:

| Class | Exit Code |
| --- | --- |
| User/input | `2` |
| Config | `10` |
| State | `11` |
| Permission | `13` |
| Authorization | `14` |
| Registry | `20` |
| Podman | `21` |
| Caddy | `22` |
| I/O | `25` |
| Process | `26` |
| Validation | `30` |
| Runtime | `50` |

Successful commands exit `0`.

## Command Surface

### `cadman init`

Creates a project config in the current working directory.

Flags:

| Flag | Behavior |
| --- | --- |
| `--yaml` | Create `cadman.yaml` instead of `cadman.toml`. |

Mutates: project config file.

### `cadman scan`

Recursively discovers Cadman project configs.

Usage:

```sh
cadman scan
cadman scan <PATHS...>
cadman scan --add
cadman scan --max-depth 8
```

Flags:

| Flag | Behavior |
| --- | --- |
| `--add` | Register discovered project-config apps in `registry.toml`. |
| `--max-depth <N>` | Override configured discovery depth. |

Mutates: `registry.toml` only when `--add` registers apps.

### `cadman containers`

Lists Podman containers visible under Cadman run-mode policy.

Flags:

| Flag | Behavior |
| --- | --- |
| `--all` | Show running and stopped containers. |
| `--running` | Show running containers only. |
| `--stopped` | Show stopped/exited containers only. |
| `--labels` | Include labels in text output. |

Read-only.

### `cadman config`

Subcommands:

| Command | Behavior |
| --- | --- |
| `cadman config path` | Show resolved `config.toml` path. |
| `cadman config show` | Show current config or runtime defaults. |
| `cadman config init` | Write default global config. |

Mutates: `config.toml` for `config init`.

### `cadman registry`

Subcommands:

| Command | Behavior |
| --- | --- |
| `cadman registry path` | Show resolved `registry.toml` path. |
| `cadman registry list` | List registered apps. |
| `cadman registry show <APP>` | Show an app by id or name. |
| `cadman registry remove <APP>` | Remove an app by id or name. |

Mutates: `registry.toml` for `registry remove`.

### `cadman add`

Registers the current project config as a project-config app.

Flags:

| Flag | Behavior |
| --- | --- |
| `--name <NAME>` | Override display name. |
| `--id <ID>` | Override app id. |

Mutates: `registry.toml`.

### `cadman remove <APP>`

Removes an app from `registry.toml` by id or name.

Mutates: `registry.toml`.

### `cadman status [APP]`

Shows runtime, diagnostics, registry, state, app, container, route, and site status. With `APP`, filters by app id or name.

Read-only.

### `cadman doctor`

Diagnoses Podman, Caddy, systemd, config, registry, state, Caddy site directory, Cadman user/group policy, run-mode access, and stale generated site files.

Read-only.

### `cadman reconcile`

Collects desired routes from labeled running Podman containers, registered project configs, and optional discovery auto-registration. It writes deterministic Cadman-managed Caddy site files, removes stale generated files, validates Caddy, reloads only when validation succeeds and files changed, updates `state.json`, and registers explicit label-sourced apps.

Mutates: `registry.toml`, generated Caddy site files, Caddy process state, and `state.json` unless `--dry-run` is used.

### `cadman self`

Subcommands:

| Command | Behavior |
| --- | --- |
| `cadman self install` | Build and optionally apply an install plan. |
| `cadman self update` | Build and optionally apply an update plan. |
| `cadman self uninstall` | Build and optionally apply an uninstall plan. |
| `cadman self healthcheck` | Report install facts and diagnostics. |

`--dry-run` reports planned lifecycle actions without mutation.

### Wrappers

| Command | Behavior |
| --- | --- |
| `cadman caddy <ARGS...>` | Run Caddy through Cadman policy. |
| `cadman podman <ARGS...>` | Run Podman through Cadman policy. |
| `cadman compose <ARGS...>` | Run Podman Compose through Cadman policy. |

Wrapper commands pass through arguments intentionally. They are controlled escape hatches, not part of reconcile’s internal orchestration path.

## Mutation Summary

Read-only commands:

```txt
cadman containers
cadman config path
cadman config show
cadman registry path
cadman registry list
cadman registry show
cadman status
cadman doctor
cadman self healthcheck
```

Mutating commands:

```txt
cadman init
cadman config init
cadman add
cadman remove
cadman registry remove
cadman scan --add
cadman reconcile
cadman self install
cadman self update
cadman self uninstall
cadman caddy
cadman podman
cadman compose
```

## Out Of Scope

Cadman v0.1.0 does not provide:

```txt
Docker runtime support
Docker socket discovery
Traefik integration
Kubernetes support
Windows support
macOS support
public API surface
public Web UI surface
required daemon/live watcher runtime
implicit sudo escalation
Caddy API route mutation as the primary write path
```
