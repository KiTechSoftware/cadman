# 🛠️ Cadman Executive Summary

**Cadman** is a Rust-based CLI for managing Podman containers, Podman Compose setups, and the Caddy web server. It simplifies local development and production orchestration through declarative `cadman.toml` configuration files, service integration, and secure domain routing.

---

## 📦 Core Commands Overview

| Command          | Arguments / Flags                                     | Description                                                       |         |                                             |
| ---------------- | ----------------------------------------------------- | ----------------------------------------------------------------- | ------- | ------------------------------------------- |
| `cadman init`    | `--interactive`, `--force`                            | Initializes a new Cadman-managed project. Creates `.cadman.yaml`. |         |                                             |
| `cadman add`     | `--path`, `--global`, `--local`                       | Registers the project in `apps.yaml`.                             |         |                                             |
| `cadman remove`  | `--name`, `--purge`, `--global`, `--local`            | Deregisters a project. Use `--purge` to remove stale references.  |         |                                             |
| `cadman purge`   | `--dry-run`, `--force`                                | Deletes apps marked as `not_found`.                               |         |                                             |
| `cadman up`      | `--detach`, `--build`, `--no-cache`, `--user` options | Starts containers or services.                                    |         |                                             |
| `cadman down`    | `--all`, `--force`, `--preserve-network`              | Stops and removes services or containers.                         |         |                                             |
| `cadman reload`  | `--force`, `--skip-caddy`, `--skip-service`           | Rebuilds and restarts services. Reloads Caddy config.             |         |                                             |
| `cadman apps`    | `--all`, `--status`, \`--output=json                  | yaml                                                              | table\` | Lists all registered apps and their status. |
| `cadman port`    | `--used`, `--free`, `--domain`, `--service`           | Shows available and used ports.                                   |         |                                             |
| `cadman info`    | `--service`, \`--output=json                          | yaml                                                              | table\` | Displays metadata from `.cadman.yaml`.      |
| `cadman logs`    | `--app`, `--tail`, `--follow`, `--level`, `--output`  | Displays logs from apps or Cadman itself.                         |         |                                             |
| `cadman update`  | `--check`, `--channel`, `--dry-run`                   | Updates the Cadman CLI (self-update or via package manager).      |         |                                             |
| `cadman scan`    | `--strict`, `--json`, `--file`, `--container`         | Uses Snyk to scan dependencies and containerfiles.                |         |                                             |
| `cadman pod`     | `--root`, `--user`, `--dry-run`, passthrough args     | Wrapper around `podman`. Respects RunMode.                        |         |                                             |
| `cadman compose` | `--root`, `--user`, `--dry-run`, passthrough args     | Wrapper around `podman-compose`. Supports compose discovery.      |         |                                             |
| `cadman caddy`   | `--config`, `--json`, `--as-root`, passthrough args   | Wrapper around Caddy CLI and API operations.                      |         |                                             |

---

## 📁 Supporting Files

* `.cadman.yaml`: Project-specific configuration (services, ports, domains)
* `apps.yaml`: Central app registry (status, path, metadata)
* `config.yaml`: Optional global/user config (logging, defaults, etc.)
