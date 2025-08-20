# Cadman Software Requirements Specification (SRS)

## Table of Contents

1. [Introduction](#1-introduction)
2. [Overall Description](#2-overall-description)
3. [Functional Requirements](#3-functional-requirements)
4. [Non-Functional Requirements](#4-non-functional-requirements)
5. [System Features and Interfaces](#5-system-features-and-interfaces)
6. [Appendices](#6-appendices)

## 1. Introduction

### 1.1 Purpose

Cadman is a Rust-based command-line tool that streamlines the management of Podman containers, Podman Compose setups, and the Caddy web server. Its goal is to simplify local and production container orchestration, service bootstrapping, and secure domain management across Linux and macOS environments.

### 1.2 Scope

Cadman supports:
- Wrapper commands for Podman, Podman Compose, and Caddy
- Project initialization (`init`) similar to `npm` or `cargo`
- Application registration and management
- System service file and domain configuration via `cadman.yaml`
- DNS-01 TLS certificate automation using Caddy and Technitium DNS
- Logging and service orchestration
- Installation via Homebrew (initially), with future support for apt and dnf

### 1.3 Definitions

- **RunMode**: Determines if Cadman runs commands as root, user, or `cadman` system user
- **apps.toml**: A registry file that tracks all Cadman-managed applications
- **state.toml**: A file storing runtime state information for Cadman
- **cadman.yaml**: Project configuration file containing container, and domain data
- **.env**: Environment variable file for service configuration
- **Podman**: A daemonless container engine for developing, managing, and running OCI containers
- **Caddy**: A powerful, enterprise-ready, open-source web server with automatic HTTPS

## 2. Overall Description

### 2.1 Product Perspective

Cadman wraps existing tools (Podman, Caddy) with added structure, configuration management, and runtime security policies. Currently supports Linux and macOS, with partial WSL support planned.
The wrapper commands (`cadman pod`, `cadman compose`, `cadman caddy`) ensure commands run in the correct context (user, root, cadman user) and provide additional logging and error handling. Using the API for Caddy and Podman where possible to avoid shelling out is a future goal. Currently, Cadman shells out to the respective CLIs.

Cadman has a daemon-like behavior for managing services, but does not run as a persistent background service unless explicitly set up by the user which is possible via system installation option during `cadman install`. It relies on systemd (Linux) or launchd (macOS) for service management.

Cadman uses YAML for project configuration (`cadman.yaml`) and a central registry (`apps.toml`) to track managed applications. It supports both global and local installations, allowing users to manage apps in user space or system-wide.

Cadman is designed to support zero-privilege execution by default, running commands as the `current user` or `cadman` user (if installed globally) unless explicitly overridden. This ensures that users can manage their applications without requiring root access, enhancing security and reducing the risk of accidental system changes.

Cadman is not a replacement for Docker or Docker Compose, but rather a slimed down version of traefik that focuses on Podman and Caddy integration. It does not support Docker or Docker Compose directly, but may add support in the future if feasible.

### 2.2 Product Functions

- Self-management: `install`, `update`, `uninstall`, `doctor`, `configure`, `ports`, `info`, `logs`
- Daemon commands: `daemon start`, `daemon stop`, `daemon reload`, `daemon status`,
- Project management: `init`, `up`, `down`, `purge`
- Application registry commands: `app add`, `app remove`, `app ls`, `app ports`, `app status`, `app logs`
- CLI wrappers: `pod`, `compose`, `caddy`

### 2.3 User Classes

- Developers
- DevOps engineers
- System administrators

### 2.4 Operating Environment

- Rust CLI
- OS: macOS, Debian/Ubuntu, Fedora (WSL support partial)
- System dependencies: Podman, Podman Compose, Caddy

### 2.5 Constraints

- Must support `cadman.yaml`, `cadman.yml` only (YAML format)
- Must support least-privilege execution via RunMode
- Must allow global and local installations
- Must not require Docker or Docker Compose (future support might be added if possible)

### 2.6 Assumptions

- `cadman.yaml` or `cadman.yml` exists per project
- Global config file may exist at `/etc/cadman/config.toml` or `$HOME/.config/cadman/config.toml`
- User has rights to write logs, config, and system services if needed
- `apps.toml` is maintained per install context (global/local)
- `state.toml` is used for runtime state management
- `--verbose` and `--debug` flags are available for detailed output

## ✅ Section 3 – Functional Requirements (Expanded)

### **FR0 – Overview**

### **FR0.1 – Global Commands**

Cadman must provide global commands for installation, configuration, and self-management.
* Must support `cadman install`, `cadman update`, `cadman uninstall`, `cadman doctor`, and `cadman configure`.
* Must allow installation via Homebrew, apt, or manual script.
* Must support self-diagnosis and configuration checks.

### **FR0.2 – Global Config File (`config.toml`)**

Reads a TOML config file from global or user location.

* Controls defaults: root path, log level, run mode, compose file fallback.

### **FR0.3 – YAML-Based Project Configuration**

`cadman.yaml` is used to define all app behavior.

* Must include `name`, `container_name`, and optional `service`, `ports`, `caddy`.
* Must support `cadman.yaml` or `cadman.yml` as valid filenames.

### **FR0.4 – Application Registry (`apps.toml`)**

Stores app metadata centrally.

* Includes `status`, `last_seen`, and file path.
* Used for autoload, validation, and purge logic.

### **FR0.5 – Status Polling**

Cadman must periodically verify app existence.

* Marks as `not_found` if project directory or `cadman.yaml` is missing.
* Logs skipped apps on reload or up.

### **FR0.6 – Port Allocation**

Cadman must assign ports automatically if not specified.

* Prefers localhost-only bindings.
* Avoids conflict with reserved or in-use ports.

### **FR0.7 – Service Inference**

If no `servicefile` is defined, Cadman must build service logic from `cadman.yaml`.

* Defaults include `WorkingDirectory`, `ExecStart`, and `User`.

### **FR0.8 – Caddyfile or JSON Generation**

Cadman generates Caddy config from YAML.

* Defaults to JSON API format.
* Can generate Caddyfile if `--caddyfile` is passed.

### **FR0.9 – Domain and TLS Setup**

If `tls: auto` or `tls: dns` is set, Cadman must configure Caddy for HTTPS.

### **FR0.10 – Autoload Skipping**

Apps marked `not_found` are ignored during lifecycle ops.

---

### **FR1 - Self-management Commands**

Cadman must provide commands for self-management, including installation, updates, and configuration management of the `config.toml`.

### **FR1.1 – Install Cadman (`cadman install`)**

Starts the Cadman daemon.

* Detects service changes and applies updates.
* Regenerates Caddy config and restarts system services if necessary.
* Respects global `auto_reload` config.

### **FR1.2 – Update Cadman (`cadman update`)**

Updates Cadman CLI binary.

* Respects install source (brew, apt, script)
* Supports `--check`, `--channel`, `--dry-run`

### **FR1.3 – Uninstall Cadman (`cadman uninstall`)**

Stops the Cadman daemon.

* Gracefully stops all running services.
* Cleans up resources and temporary files.
* Supports `--force` to immediately stop all services.

### **FR1.4 – Show/Fix Cadman Health Status (`cadman doctor`)**

Displays the health status of the Cadman installation.

* Shows whether the cadman installation is healthy or has issues.
* Displays if `config.toml` is present and valid.
* Displays if dependencies like Podman, Podman Compose, and Caddy are installed and accessible.
* Provides a summary of detected issues and their severity including suggestions for fixing common issues.
* Supports `--verbose` for detailed output.
* Supports `--fix` to attempt automatic repairs.

### **FR1.5 – Configure Cadman (`cadman configure`)**

Updates the global configuration file (`config.toml`).
* Allows modification of default paths, logging, and run modes.
* Supports `--set`, `--get`, and `--reset` commands.
* Validates changes and provides feedback on success or failure.

---

### **FR2.0 - Daemon Management Commands**

Cadman must provide commands to manage the Cadman daemon, including starting, stopping, reloading, and checking status.

### **FR2.1 – Start Cadman Daemon (`cadman start`)**

Starts the Cadman daemon.

* Detects service changes and applies updates.
* Regenerates Caddy config and restarts system services if necessary.
* Respects global `auto_reload` config.

### **FR2.2 – Stop Cadman Daemon (`cadman stop`)**

Stops the Cadman daemon.

* Gracefully stops all running services.
* Cleans up resources and temporary files.
* Supports `--force` to immediately stop all services.

### **FR2.3 – Reload Cadman Daemon (`cadman reload`)**

Rebuilds and restarts the Cadman daemon.

* Detects service changes and applies updates.
* Regenerates Caddy config and restarts system services if necessary.
* Respects global `auto_reload` config.

### **FR2.4 – Show Cadman Daemon Status (`cadman status`)**

Displays the status of the Cadman daemon.

* Shows whether the daemon is running or stopped.
* Displays active services and their health status.
* Supports `--verbose` for detailed output.

---

### **FR3.0 – Project Management Commands**

Cadman must provide commands to manage projects, including initialization, starting, stopping, and purging services.
* Must support `cadman init`, `cadman up`, `cadman down`, and `cadman purge`.
* Must read and write to `cadman.yaml` for project configuration.
* Must support service definitions, ports, and Caddy configuration within `cadman.yaml`.

#### **FR3.1 – Project Configuration (`cadman.yaml`)**

Cadman must use a YAML file (`cadman.yaml` or `cadman.yml`) to define project settings.
* Must include `name`, `container_name`, and optional `service`, `ports`, `caddy` sections.
* Supports `cadman.yaml` or `cadman.yml` as valid filenames.
* Must support `cadman.yaml` in the current directory or a specified path.
* Must validate the presence of required fields and warn if missing.
* Must allow optional fields for service configuration, ports, and Caddy settings.
* Must support comments using `#` for documentation within the YAML file.
* Must allow for multiple services to be defined within the same `cadman.yaml`.
* Must support environment variable substitution for dynamic values.


#### **FR3.2 – Project Initialization (`cadman init`)**

Bootstraps a new Cadman-compatible project.

* Creates `cadman.yaml` with service, domain, and port info.
* Supports optional service and Caddy config generation.
* Interactive or non-interactive modes.


#### **FR3.3 – Start Project Service (`cadman up`)**

Cadman must start containers or services defined in the project's `cadman.yaml`.

* Starts all services if no service name is provided.
* Supports `compose_file` or `container_file` execution.
* Supports flags: `--detach`, `--build`, `--no-cache`, `--user`, `--root`, `--cadman`.


#### **FR3.4 – Stop Project Service (`cadman down`)**

Stops and removes all containers or services defined in the project's `cadman.yaml`.

* Accepts `--all`, `--force`, and `--preserve-network`.
* Stops either all or a specified service.


#### **FR3.5 – Purge Project Service (`cadman purge`)**

Rebuilds and restarts services.

* Detects service changes and applies updates.
* Regenerates Caddy config and restarts system services if necessary.
* Respects global `auto_reload` config.

---

### **FR4 – Service Management Commands**


#### **FR4.1 – List Registered Apps (`cadman ls`)**

Lists all registered applications from the registry.

* Supports filters: `--all`, `--status`, `--service`, `--output`.

#### **FR4.2 – Show Ports (`cadman ports`)**

Displays all ports in use by registered apps.

* Can show `--used`, `--free`, `--domain`, `--service`.

#### **FR4.3 – Show Cadman Info (`cadman info`)**

Displays metadata for a project or service.

* Reads `cadman.yaml`.
* Supports `--service` and `--output`.

#### **FR4.4 – Show Cadman Logs (`cadman logs`)**

Displays logs for apps or Cadman itself.

* Supports: `--app`, `--tail`, `--follow`, `--level`, `--output`

---

### **FR5 – Wrapper Commands**

Cadman must provide wrapper commands for Podman, Podman Compose, and Caddy to ensure commands run in the correct context and with appropriate logging.
* Must support `cadman pod`, `cadman compose`, and `cadman caddy`.
* Must allow execution of Podman and Podman Compose commands with additional flags.

#### **FR5.1 – Wrapper: Podman (`cadman pod`)**

Executes a `podman` command in the correct RunMode.

* Default: `cadman` user
* Supports `--root`, `--user`, `--dry-run`, `--log`, `--verbose`

#### **FR5.2 – Wrapper: Podman Compose (`cadman compose`)**

Executes a `podman-compose` command.

* Resolves compose file automatically.
* Supports same flags as `cadman pod`.

#### **FR5.3 – Wrapper: Caddy (`cadman caddy`)**

Runs Caddy commands via CLI or API.

* Supports `--config`, `--json`, `--as-root`, etc.
* Can reload or validate configs.

---

### **FR6 – Application Registration Commands**

#### **FR6.1 – Register Application (`cadman app add`)**

Adds the current or specified project to the registry.

* Updates `apps.toml`.
* Validates existence of `cadman.yaml`.
* Supports `--path`, `--global`, `--local`.

#### **FR6.2 – Deregister Application (`cadman app remove`)**

Removes an app from the registry without deleting files.

* Marks app as `deleted`.
* Supports `--name`, `--purge`, `--global`, `--local`.

### **FR6.3 – Purge Missing Apps (`cadman app purge`)**

Removes all apps marked `not_found`.

* Scans registry and deletes stale entries.
* Supports `--dry-run`, `--force`.

---

## 4. Non-Functional Requirements

### NFR1 – Secure Run Context
Cadman must default to executing commands as the `cadman` user unless explicitly overridden.

### NFR2 – Format Support
Only YAML (`cadman.yaml` or `cadman.yml`) is supported for project configuration. TOML, JSON, and INI formats are not supported (currently).

### NFR3 – Output Consistency
All CLI commands that generate output must support `--output json|yaml|table` for both human and machine-readable formats.

### NFR4 – Performance
- `cadman apps`, `cadman ports`, and `cadman info` must return results in under 500ms for <20 apps.
- Lifecycle commands (`up`, `down`, `reload`) should begin execution within 1 second.

### NFR5 – Registry Safety
- If `apps.toml` is missing, Cadman must create it without error.
- Changes to the registry must use file locking to prevent race conditions.

### NFR6 – Logging
- Logs must be written to file by default (`~/.cadman/logs` or `/var/log/cadman`).
- Support `log_output` values: `file`, `stdout`, `stderr`, `stacked`.
- Users may override this using global config or `CADMAN_LOG_OUTPUT` env var.

### NFR7 – Port Management
- Cadman must not assign duplicate or in-use ports.
- Must check if port is already bound before launching services.

### NFR8 – Config Precedence
- Command-line flags override environment variables
- Environment variables override global config
- Global config overrides built-in defaults

### NFR9 – File Generation Rules
- Cadman must not overwrite existing `cadman.yaml`, `apps.toml`, or servicefiles unless `--force` is passed.

### NFR10 – Global vs Local Mode
- If run as `cadman` user or root, use global paths
- If run as normal user, default to user paths unless `--global` is explicitly passed

### NFR11 – Error Handling
- All commands must return non-zero exit codes on failure
- All commands must provide human-readable and actionable error messages

### NFR12 – Caddy Reload Behavior

- Cadman must use Caddy's API or CLI to reload changes without downtime
- Must validate Caddyfile or JSON config before reload

### NFR13 – CI/CD Friendly

- Commands like `scan`, `info`, and `apps` must work in non-interactive environments
- Logging must support JSON format for CI log parsing

### NFR14 – Safe Defaults

- If `cadman.yaml` is incomplete or missing fields, Cadman must use safe, opinionated defaults and warn the user

## 5. System Features and Interfaces

### 5.1 Global Configuration

* File: `config.toml`
* Locations:

  * Global: `/etc/cadman/config.toml`
  * User: `$HOME/.config/cadman/config.toml`
* Sample fields:

  ```toml
  [defaults]
  run_mode = "cadman"
  log_output = "stacked"
  auto_reload = true
  project_paths = ["/opt/apps"]
  compose_preference = ["podman-compose", "docker-compose"]
  
  ```

### 5.2 Project Configuration

* File: `cadman.yaml` (required per project)
* Fields:

  ```yaml
  name: my-app
  container_name: my-app-container
  ports:
    - 8080
  caddy:
    domains:
      - myapp.local
    tls: auto
  service:
    enabled: true
    user: cadman
    exec_start: podman-compose up
  ```

### 5.3 Application Registry

* File: `apps.toml` (per Cadman install)
* Contents:

  ```toml
  [[apps]]
  name = "my-app"
  path = "/opt/apps/my-app"
  status = "active"
  last_seen = "2025-07-05T12:00:00Z"

  [[apps]]
  name = "stale-app"
  path = "/opt/apps/stale"
  status = "not_found"
  last_seen = "2025-07-01T14:00:00Z"
  ```

### 5.4 Podman Integration

* Commands run as `cadman` user unless overridden.
* Supports container discovery, logging, lifecycle.

### 5.5 Podman Compose Integration

* Compose file fallback order:

  1. `podman-compose.yaml`
  2. `docker-compose.yaml`
* Used in `init`, `up`, `reload`.

### 5.6 Caddy Integration

* Generates `caddy_config.json` or `Caddyfile` from `cadman.yaml`
* Config reloads automatically on `reload` or `up`.

### 5.7 Logging

* Default: write to `~/.cadman/logs/` or `/var/log/cadman/`
* Configurable via global config:

  * `log_output: stdout`, `stderr`, `file`, or `stacked`

## 6. Appendices

### 6.1 Sample `cadman.yaml`

```yaml
name: blog-site
container_name: blog-container
ports:
  - 3000
caddy:
  domains:
    - blog.local
  tls: auto
service:
  enabled: true
  user: cadman
  exec_start: podman-compose up
  exec_stop: podman-compose down
```

---

### 6.2 Sample `config.toml`

```yaml
root_path: /opt/apps
default_run_mode: cadman
log_output: file
compose_preference:
  - podman-compose
  - docker-compose
auto_reload: true
```

---

### 6.3 Sample `apps.toml`

```toml
[[apps]]
name = "blog-site"
path = "/opt/apps/blog-site"
status = "active"
last_seen = 2025-07-05T13:45:00Z

[[apps]]
name = "old-api"
path = "/opt/apps/old-api"
status = "not_found"
last_seen = 2025-07-01T18:10:00Z
```

---

### 6.4 Environment Variables

| Variable             | Description                                     | Default     |
| -------------------- | ----------------------------------------------- | ----------- |
| `CADMAN_LOG_OUTPUT`  | Logging destination                             | `file`      |
| `CADMAN_ROOT_PATH`   | Root directory for apps                         | `/opt/apps` |
| `CADMAN_RUN_MODE`    | Run mode for command execution                  | `cadman`    |
| `TECHNITIUM_API_KEY` | API key for DNS challenge automation (optional) | *(none)*    |

---

### 6.5 External Dependencies

| Tool           | Purpose                       |
| -------------- | ----------------------------- |
| Podman         | Container management          |
| Podman Compose | Multi-container orchestration |
| Caddy          | Web server with auto TLS      |
| Systemd        | Service integration (Linux)   |

## 7. DevOps and CI/CD Requirements

### 7.1 GitHub Actions Workflow

Cadman must include a GitHub Actions pipeline to validate, build, and secure code contributions to the `main` branch.

#### Pipeline Stages:

| Stage            | Purpose                                                          |
| ---------------- | ---------------------------------------------------------------- |
| `lint`           | Ensure formatting and lint compliance for Rust and shell scripts |
| `build`          | Compile the Cadman CLI for Linux and macOS targets               |
| `test`           | Run unit and integration tests (where available)                 |

#### Conditions:

* Runs on all PRs targeting `main`
* Fails the workflow if vulnerabilities are found with high/critical severity (unless explicitly approved)

### 7.2 Protection Rules

* Branch protection for `main` must require successful completion of the GitHub Actions workflow
* Manual approval required to override failed security scans
* Force pushes to `main` must be disabled
