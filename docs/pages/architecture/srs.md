# Cadman Software Requirements Specification (SRS)

> **Audience:** engineers, PMs, ops.

## Table of Contents

1. [Introduction](#introduction)
2. [Overall Description](#overall-description)
3. [Functional Requirements](#functional-requirements)

   * [FR-C Core](#fr-c--core-self-management--project-lifecycle--registry--ports--caddy)
   * [FR-D Daemon](#fr-d--daemon-management)
   * [FR-P Project](#fr-p--project-management)
   * [FR-A App Registry](#fr-a--application-registry)
   * [FR-W Wrappers](#fr-w--wrappers)
   * [FR-S Self-Introspection](#fr-s--self-introspection)
4. [Non-Functional Requirements](#non-functional-requirements)
5. [System Features & Interfaces](#system-features-and-interfaces)

   * [CLI Conventions & Surface](#cli-conventions--surface)
   * [File Interfaces](#file-interfaces)
   * [Podman / Podman Compose](#podman--podman-compose)
   * [Caddy](#caddy)
   * [Exit Codes](#exit-codes)
6. [Concurrency & Race-Condition Avoidance](#concurrency--race-condition-avoidance) 🔐
7. [Configuration](#configuration)
8. [Appendices](#appendices)

---

## Introduction

### Purpose

Cadman is a **Rust CLI** that streamlines **Podman**, **Podman Compose**, and **Caddy** operations for local and production use on **Linux** and **macOS**. It emphasizes **least privilege**, **predictable configuration**, and **clean UX** for app lifecycle, ports, and reverse proxy/TLS.

### Scope

* Wrappers: `pod`, `compose`, `caddy`
* Project lifecycle: `init`, `up`, `down`, `purge`
* App registry & status: `app add|remove|ls|info|status|ports|logs`
* Self-management: `install`, `update`, `uninstall`, `doctor`, `configure`, `info`, `ports`, `logs`
* Daemon: `daemon start|stop|reload|status`

### Definitions & Acronyms

* **RunMode**: `cadman` (default) | `user` | `root`
* **apps.toml**: registry of managed apps
* **state.toml**: runtime state (ephemeral-ish)
* **cadman.yaml**: per-project configuration
* **ULID**: time-sortable unique ID for apps
* **Caddy**: reverse proxy with auto-TLS

---

## Overall Description

### Product Perspective

Cadman **wraps CLIs** (future APIs possible), applies **RunMode** rules, and uses **systemd/launchd** for service control. It can be installed **globally** (system paths) or **per-user** (home paths). Security is **least-privilege by default**.

**Not a Docker replacement**; **no Docker/Docker Compose** support today.

### Operating Environment

*Application Type:* Rust binary; 

*Platforms:* macOS, Debian/Ubuntu, Fedora (WSL partial);

*Dependencies:* requires Podman, Podman Compose, Caddy.

### Constraints

* Project config: `cadman.yaml` or `cadman.yml` only
* Global/local installs must co-exist cleanly
* No auto-escalation; explicit escalation only

### Assumptions

* One `cadman.yaml` per project root
* Registry lives per install scope
* Users can write logs/config in their scope

---

## Functional Requirements

> Each FR shows **Purpose → Behavior → Flags** and cross-links to examples when relevant.

### FR-C — Core (Self-Management • Project Lifecycle • Registry • Ports • Caddy)

**FR-C1 – Install Cadman (`cadman install`)** 📦
**Purpose:** Install or finalize Cadman in current scope.
**Behavior:**

* Create required dirs/files (config, logs, registry) if missing.
* Optionally create `cadman` user/group on Linux for global install.
* Validate dependencies; warn if missing; exit `3` when hard required.
  **Flags:** `--global | --user`, `--dry-run`, `--quiet`
  **Errors:** Missing perms → `3`; partial init but usable → warn + `0`.

**FR-C2 – Update (`cadman update`)** ⬆️
**Purpose:** Update binary via original channel.
**Behavior:** Detect channel, fetch, verify, replace atomically.
**Flags:** `--check`, `--channel stable|nightly`, `--dry-run`

**FR-C3 – Uninstall (`cadman uninstall`)** 🧹
**Purpose:** Remove Cadman from scope.
**Behavior:** Remove binaries, optional data (`--purge-data`), leave apps untouched unless `--force`.
**Flags:** `--purge-data`, `--force`, `--dry-run`

**FR-C4 – Doctor (`cadman doctor`)** 🩺
**Purpose:** Diagnose env, dependencies, config health.
**Behavior:** Prints checks + suggested fixes; optionally `--fix`.
**Flags:** `--verbose`, `--fix`, `--output table|json|yaml`

**FR-C5 – Configure (`cadman configure`)** ⚙️
**Purpose:** Get/set global config.
**Behavior:** `--set key=value`, `--get key`, `--reset`. Atomic write w/ fsync.
**Flags:** `--global|--user`, `--output`

**FR-C6 – Project Lifecycle (`cadman init|up|down|purge`)** 🚀

* **init**: Create `cadman.yaml` (idempotent), optionally derive service and Caddy snippets from in-memory defaults when file absent.
* **up**: Start services/containers; allocate ports; apply Caddy; zero-downtime reload.
* **down**: Stop services; keep reservations unless `--release-ports`.
* **purge**: Remove containers/volumes/artifacts; **NEVER** delete source unless `--force`.
  **Flags (common):** `--debug`, `--verbose`, `--output`, `--dry-run`

**FR-C7 – Port Allocation** 🔌
**Purpose:** Auto-allocate ports when unspecified; avoid conflicts.
**Behavior:** Single writer lock on allocator; OS-level probe; atomic reservation. Mid-lifecycle conflict → see [Race Avoidance](#concurrency--race-condition-avoidance).
**Flags:** `--range A-B`, `--prefer-localhost`

**FR-C8 – Caddy Config** 🧩
**Purpose:** Generate/apply Caddy JSON (default) or Caddyfile on `--caddyfile`.
**Behavior:** Validate before apply; reload is atomic; roll back on failure.
**Flags:** `--dry-run`, `--caddyfile`

**FR-C9 – Output & Verbosity** 🗣️
**Purpose:** Consistent outputs.
**Behavior:** Read-only cmds support `--output table|json|yaml`; global `--verbose|--debug|--quiet`.

---

### FR-D — Daemon Management

**FR-D1 – Start (`cadman daemon start`)** ▶️
Start managed background unit in scope (systemd/launchd). Respect RunMode.

**FR-D2 – Stop (`cadman daemon stop`)** ⏹️
Graceful stop; timeout → escalate; return `5` if still running.

**FR-D3 – Reload (`cadman daemon reload`)** 🔁
Reload config, re-evaluate services, reapply Caddy. Idempotent.

**FR-D4 – Status (`cadman daemon status`)** 📊
Return running state + health summary. `--output` formats.

---

### FR-P — Project Management

**FR-P1 – Config (`cadman.yaml`)**
**Purpose:** Declarative project config.
**Behavior:** Validate required fields; env-subst; support `env_file`; multiple services/containers.
See [Appendix 7.1](#71-sample-cadmanyaml).

**FR-P2 – Init (`cadman init`)** ✨
**Purpose:** Bootstrap safely (re-runnable).
**Behavior:** If file exists, merge non-destructively unless `--force`. Optional service/Caddy snippets derived from current YAML; if YAML absent, use in-memory defaults.
**Flags:** `--with-service`, `--with-caddy`, `--force`, `--non-interactive`

**FR-P3 – Up/Down/Purge**
**Purpose:** Lifecycle wrapper w/ RunMode & logs.
**Behavior:** Compose/Container selection by preference; respect `working_directory`, `after/requires`. Fail predictable with exit codes.

---

### FR-A — Application Registry

**FR-A1 – Add (`cadman app add`)** ➕
**Purpose:** Register project in `apps.toml`.
**Behavior:** Validate path + `cadman.yaml`; create **ULID**; lock registry; write atomically. Missing path/YAML → exit `4`.
**Flags:** `--path`, `--global|--local`

**FR-A2 – Remove (`cadman app remove`)** ➖
Mark deleted; `--purge` removes entry & port reservations.

**FR-A3 – List (`cadman app ls`)** 📜
Filter by `--status`, `--output`. Sorted, stable columns.

**FR-A4 – Info (`cadman app info`)** 🧠
Static view: registry + resolved config + derived service + effective Caddy snippet (secrets redacted).
➡️ See [Example JSON](#74-example-outputs-json).

**FR-A5 – Status (`cadman app status`)** 🩺
Runtime: containers, service state, Caddy route health.
➡️ See [Example JSON](#74-example-outputs-json).

**FR-A6 – Ports (`cadman app ports`)** 🔎
Show reserved/requested vs observed; highlight mismatches.

**FR-A7 – Logs (`cadman app logs`)** 📓
`--follow`, `--since`, `--tail`, `--level`; merges container + service + Caddy (source tag included).

---

### FR-W — Wrappers

**FR-W1 – Podman (`cadman pod`)** 🧰
Pass-through to `podman` with RunMode + logging + cwd rules.
**Flags:** `--root`, `--user`, `--cadman` (default), `--dry-run`, `--verbose`.

**FR-W2 – Compose (`cadman compose`)** 🧩
Locate compose file in priority: `podman-compose.yaml` → `docker-compose.yaml` (if allowed by config). Same flags as `pod`. (Docker compose support **may** be future; doc current scope as Podman Compose.)

**FR-W3 – Caddy (`cadman caddy`)** 🪄
Run CLI/API; validate configs; `--json`/`--config`. RunMode default `cadman`.

---

### FR-S — Self-Introspection

**FR-S1 – Info (`cadman info`)** 🧾
Show effective global config after precedence. `--output` formats.
➡️ See [Config precedence diagram](#config-precedence-diagram).

**FR-S2 – Logs (`cadman logs`)** 🗂️
Cadman’s own logs; `--follow|--since|--tail|--level`.

**FR-S3 – Ports (`cadman ports`)** 🧮
Allocator state: configured ranges, reserved per app, **free**, conflicts.
**Flags:** `--used`, `--free`, `--range A-B`, `--domain`, `--service`.
➡️ See [Example JSON](#74-example-outputs-json).

---

## Non-Functional Requirements

**Security (NFR-1)**

* Default RunMode: `cadman`.
* No escalation without explicit flag (`--root` or RunMode override).
* On Linux global install, run via `cadman` user + group; add caller to `cadman` group (opt-in).
* Sensitive values redacted in outputs/logs.

**Formats (NFR-2)**

* Read-only commands support `--output table|json|yaml`.
* JSON includes `"schema_version": "1"` (see schema policy below).

**Performance (NFR-3)**

* `apps|ports|info` < **500 ms** for ≤ 20 apps.
* Lifecycle start ≤ **1 s**.

**Registry Safety (NFR-4)**

* Create `apps.toml` if missing.
* File locks (advisory) + atomic writes (temp → fsync → rename).

**Logging (NFR-5)**

* Default file logs (`~/.cadman/logs` or `/var/log/cadman`).
* Configurable via config/env. Log rotation atomic (see race section).

**Port Management (NFR-6)**

* No duplicates; check **OS socket** + **registry**.
* Mid-lifecycle conflict policy in [Race Avoidance](#concurrency--race-condition-avoidance).

**Precedence (NFR-7)**
CLI > ENV > Global config > Built-ins.
➡️ See [diagram](#config-precedence-diagram).

**No Clobber (NFR-8)**
Never overwrite project files without `--force`.

**Errors (NFR-9)**

* Non-zero exit codes on failure.
* Message MUST include: summary, cause (if known), next step.

**Zero-Downtime Caddy (NFR-10)**

* Validate then reload; rollback on failure.

**CI/CD Friendly (NFR-11)**

* Non-interactive mode friendly; machine-parsable logs.

**Atomicity (NFR-12)**

* Registries/config/log index updates are **atomic** with fsync.

**Identity (NFR-13)**

* `app_id` is a ULID.

**Schema Versioning (NFR-14)**

* Add `schema_version` to JSON outputs.
* Backward-compatible additions allowed in same major.
* Breaking changes bump major & document in release notes.

---

## System Features and Interfaces

### CLI Conventions & Surface

**Global flags:** `--output`, `--verbose`, `--debug`, `--quiet`, `--dry-run`
**Selection:** `--id | --name | --path` (priority: id > name > path)
**Commands:**

* **Self-management:** `install|update|uninstall|doctor|configure|info|logs|ports`
* **Daemon:** `daemon start|stop|reload|status`
* **Project:** `init|up|down|purge`
* **App:** `app add|remove|ls|info|status|ports|logs`
* **Wrappers:** `pod|compose|caddy`

### File Interfaces

* Global config: `/etc/cadman/config.toml` (Linux), `$HOME/.config/cadman/config.toml` (user)
* Registry: `apps.toml` (per scope)
* Project: `cadman.yaml` / `cadman.yml`
* Runtime: `state.toml`
* Logs: `~/.cadman/logs` or `/var/log/cadman`

### Podman / Podman Compose

* Default RunMode: `cadman`
* Compose detection: `podman-compose.yaml` → `docker-compose.yaml` (only if enabled)
* Pass-through unknown flags; capture stdout/stderr; map exit codes

### Caddy

* Generate JSON by default; Caddyfile with `--caddyfile`.
* Validate → Apply → Zero-downtime reload. Rollback on failure.

### Exit Codes

* `0` success
* `1` usage error (bad flags/args)
* `2` not found (app/path/resource)
* `3` dependency missing/permission denied
* `4` validation error (config invalid)
* `5–9` internal error (unexpected but handled)
* `10–19` external process failure (podman/compose/caddy non-zero)
* `20–29` concurrency/race/timeouts (lock contention, mid-lifecycle conflicts)

---

## Concurrency & Race-Condition Avoidance

> **Goal:** Safe concurrent CLI runs, CI runners, and background daemon ops.

### Principles

* **Single-writer** per shared resource (registry, allocator).
* **Advisory file locks** (+ retry/backoff) around critical sections.
* **Atomic writes**: temp file → `fsync()` → `rename()` → `fsync(dir)`.
* **OS-level reality checks**: verify sockets/filesystem beyond registry.
* **Idempotency**: re-run safe; partial ops detectable and recoverable.

### Resources & Strategies

1. **Registry (`apps.toml`)**

   * Lock file: `${scope}/apps.toml.lock` (exclusive when writing; shared for read if needed).
   * Steps: read → mutate → write temp → `fsync` → `rename`.
   * On contention: exponential backoff (up to N attempts) then exit `20` with hint.

2. **Port Allocator**

   * Lock file: `${scope}/ports.lock`.
   * Allocation: scan registry → probe OS → reserve → write atomically.
   * Mid-lifecycle conflict (port stolen after up):

     * Detect on health check; **fail fast** with exit `21` unless `--auto-retry`.
     * If `--auto-retry`, attempt **one** alternative port in same range, rewrite Caddy, reload, and report mapping change.

3. **Caddy Reload**

   * Validate in memory → POST to API → verify live config → commit.
   * If failure, **rollback** to prior config; do not change registry.

4. **Compose Up/Down**

   * Lock per project: `${project}/.cadman.lock`.
   * Prevent overlapping up/down/purge.
   * If lock held > timeout, show owner PID (if stored) and exit `22`.

5. **Log Rotation**

   * Use `rename()` for rotation; writers reopen on `SIGUSR1` or periodic check.
   * Avoid truncation races by writing exclusively and reopening post-rotate.

6. **`state.toml` Updates**

   * Same atomic write pattern; tolerate missed updates (non-fatal).
   * Readers handle partial or absent keys gracefully.

7. **RunMode Escalation**

   * Never escalate implicitly. If `--root` requested and lock is held by non-root op, exit `23` with guidance to retry or wait.

---

## Configuration

### Config Precedence Diagram

```
+-------------------------+
| Built-in Defaults       |
+------------+------------+
             |
             v
+-------------------------+
| Global Config           |
| /etc or $HOME/.config   |
+------------+------------+
             |
             v
+-------------------------+
| Environment Variables   |
| CADMAN_*                |
+------------+------------+
             |
             v
+-------------------------+
| CLI Flags               |
| (--output, --debug, ..) |
+-------------------------+

Effective config is resolved bottom-up (CLI wins).
```

### RunMode Decision Tree

```
Start
 |
 |-- global install? ---- yes ---- user in 'cadman' group? -- yes --> RunMode=cadman
 |                                            |
 |                                            no
 |                                            v
 |                                      explicit --user? -- yes --> RunMode=user
 |                                            |
 |                                            no
 |                                            v
 |                                       explicit --root? -- yes --> RunMode=root
 |                                            |
 |                                            no
 |                                            v
 |---------------------------------------> RunMode=cadman
 |
 no (user install)
 |
 |-- explicit --root? ---- yes --> RunMode=root
 |-- explicit --user? ---- yes --> RunMode=user
 |                                (default) --> RunMode=user
```

---

## Appendices

### 7.1 Sample `cadman.yaml`

```yaml
name: cadman-example
description: A Cadman project

containers:
  default:
    name: blog-container
    restart: always
    detached: true
    use_compose: false
    image: ghcr.io/kitechsoftware/wordpress:latest
    env_file:
      - .env
    environment:
      - UID=$(id -u)
      - GID=$(id -g)
      - WORDPRESS_DB_HOST=${DB_HOST}
      - WORDPRESS_DB_USER=${DB_USER}
      - WORDPRESS_DB_PASSWORD=${DB_PASSWORD}
      - WORDPRESS_DB_NAME=${DB_NAME}
      - WORDPRESS_TABLE_PREFIX=${DB_PREFIX}
      - WORDPRESS_AUTH_KEY=${AUTH_KEY}
      - WORDPRESS_SECURE_AUTH_KEY=${SECURE_AUTH_KEY}
      - WORDPRESS_LOGGED_IN_KEY=${LOGGED_IN_KEY}
      - WORDPRESS_NONCE_KEY=${NONCE_KEY}
      - WORDPRESS_AUTH_SALT=${AUTH_SALT}
      - WORDPRESS_SECURE_AUTH_SALT=${SECURE_AUTH_SALT}
      - WORDPRESS_LOGGED_IN_SALT=${LOGGED_IN_SALT}
      - WORDPRESS_NONCE_SALT=${NONCE_SALT}
    ports:
      - "127.0.0.1:${BLOG_PORT:-1081}:80"
    volume:
      - $(pwd)/public:/app/public:Z

  api:
    name: api-container
    working_directory: /opt/apps/api
    detached: true
    use_compose: true
    compose_file: podman-compose.yaml
    env_file:
      - .api.env
    restart: always
    user: cadman
    group: cadman
    ports:
      - "127.0.0.1:${API_PORT:-1080}:80"
    exec_start: up.sh
    exec_stop: down.sh
    after:
      - network.target
    requires:
      - network.target

caddy:
  email: example@example.com
  tls: auto
  routes:
    - host: 
        - blog.local
        - www.blog.local
      log: stdout
      proxy: 127.0.0.1:${BLOG_PORT:-1081}
    - host: api.blog.local
      tls: internal
      email: api@example.com
      proxy: 127.0.0.1:${API_PORT:-1080}
```

### 7.2 Sample `config.toml`

```toml
[global]
debug = false
verbose = false
quiet = false
reload = true
output = "json"

[mode]
user = "cadman"
system = true

[projects]
reload = true
paths = ["/var/apps", "/opt/apps", "/mnt/apps"]

[ports]
ranges = ["3000-3999","5000-5999"]
prefer_localhost = true
auto_allocate = true

[logs]
level = "info"
size = 100
output = "file"

[preference]
auto = true

[preference.container]
podman = true
docker = true

[preference.compose]
podman_compose = true
docker_compose = true

[service.caddy]
installed = true
managed = true
user = false

[service.podman]
installed = true
managed = true
user = true

[service.podman_compose]
installed = true
managed = true
user = true
```

### 7.3 Sample `apps.toml`

```toml
[[apps]]
app_id = "cadman-example-01"
name = "cadman-example"
path = "/opt/apps/cadman-example"
status = "active"
last_seen = 2025-07-05T13:45:00Z

[[apps]]
app_id = "stale-app-01"
name = "stale-app"
path = "/opt/apps/stale"
status = "not_found"
last_seen = "2025-07-01T14:00:00Z"
```

### 7.4 Example Outputs (JSON)

**`cadman app info --id 01J3…`**

```json
{
  "schema_version": "1",
  "app_id": "cadman-example-01",
  "name": "cadman-example",
  "path": "/opt/apps/cadman-example",
  "status": "active",
  "last_seen": "2025-07-05T13:45:00Z",
  "config": {
    "resolved": {
      "containers":  {
        "default": {
          "id": "cadman-example-01-default",
          "name": "blog-container",
          "image": "ghcr.io/kitechsoftware/wordpress:latest",
          "restart": "always",
          "detached": true,
          "use_compose": false,
          "env_file": [".env"],
          "environment": {
            "UID": "$(id -u)",
            "GID": "$(id -g)",
            "WORDPRESS_DB_HOST": "${DB_HOST}",
            "WORDPRESS_DB_USER": "${DB_USER}",
            "WORDPRESS_DB_PASSWORD": "${DB_PASSWORD}",
            "WORDPRESS_DB_NAME": "${DB_NAME}",
            "WORDPRESS_TABLE_PREFIX": "${DB_PREFIX}",
            "WORDPRESS_AUTH_KEY": "${AUTH_KEY}",
            "WORDPRESS_SECURE_AUTH_KEY": "${SECURE_AUTH_KEY}",
            "WORDPRESS_LOGGED_IN_KEY": "${LOGGED_IN_KEY}",
            "WORDPRESS_NONCE_KEY": "${NONCE_KEY}",
            "WORDPRESS_AUTH_SALT": "${AUTH_SALT}",
            "WORDPRESS_SECURE_AUTH_SALT": "${SECURE_AUTH_SALT}",
            "WORDPRESS_LOGGED_IN_SALT": "${LOGGED_IN_SALT}",
            "WORDPRESS_NONCE_SALT": "${NONCE_SALT}"
          },
          "ports": ["8000:80"],
          "volume": ["$(pwd)/public:/app/public:Z"]
        },
        "api": {
          "id": "cadman-example-01-api",
          "name": "api-container",
          "working_directory": "/opt/apps/api",
          "detached": true,
          "use_compose": true,
          "compose_file": "podman-compose.yaml",
          "restart": "always",
          "user": "cadman",
          "group": "cadman",
          "env_file": [".api.env"],
          "ports": ["8000:80"],
          "exec_start": "up.sh",
          "exec_stop": "down.sh",
          "after": ["network.target"],
          "requires": ["network.target"]
        }
      },
      "ports": [3000, 8000],
      "caddy": {
        "email": "example@example.com",
        "tls": "auto",
        "routes": [
          {
            "host": ["blog.local", "www.blog.local"],
            "log": "stdout",
            "proxy": "http://localhost:3000"
          },
          {
            "host": ["api.blog.local"],
            "log": "stdout",
            "proxy": "http://localhost:8000"
          }
        ]
      }
    },
    "source_files": ["cadman.yaml"]
  }
}
```

**`cadman app status --name blog-site`**

```json
{
  "schema_version": "1",
  "app_id": "blog-site-02",
  "auto_reload": true,
  "containers": [
    {"name": "blog-container", "restart": true, "state": "running", "health": "healthy", "since": "2025-08-10T16:12:03Z"}
  ],
  "caddy": {
    "routes": [
      {"host": ["blog.local", "www.blog.local"], "status": "ok"},
      {"host": ["api.blog.local"], "status": "ok"}
    ]
  }
}
```

**`cadman ports --free --range 3000-3999`**

```json
{
  "schema_version": "1",
  "ranges": ["3000-3999"],
  "free_ports": [3001,3002,3005,3010],
  "reserved": [{"port": 3000, "app_id": "blog-site-02", "name": "blog-site"}],
  "conflicts": []
}
```

### 7.5 Sample `state.toml`

```toml
[blog-site-02]
container = "cadman-example-01-default"
status = "running"
health = "healthy"
started = "2025-08-10T16:12:03Z"
caddy_routes = ["blog.local", "www.blog.local"]
ports = [3000, 8000]
caddy_status = "ok"
caddy_route_status = "ok"
caddy_route_health = "healthy"
last_seen = "2025-08-10T16:12:03Z"
```

### 7.6 Environment Variables

| Variable             | Description                        | Default               |
| -------------------- | ---------------------------------- | --------------------- |
| CADMAN\_LOG\_OUTPUT  | Logging destination                | `file`                |
| CADMAN\_ROOT\_PATH   | Root app search paths              | `/opt/apps,/mnt/apps` |
| CADMAN\_RUN\_MODE    | `cadman` \| `user` \| `root`       | `cadman` (global)     |
| CADMAN\_PORT\_RANGES | Port ranges                        | `3000-3999,5000-5999` |
| CADMAN\_DEBUG        | Enable debug                       | `false`               |
| CADMAN\_VERBOSE      | Verbose output                     | `false`               |
| CADMAN\_QUIET        | Suppress non-error output          | `false`               |
| CADMAN\_AUTO\_RELOAD | Auto-reload Caddy on config change | `true`                |
| CADMAN\_CADDY\_EMAIL | Email for Caddy TLS                | `example@example.com` |
| CADMAN\_CADDY\_TLS   | Caddy TLS mode                     | `auto`                |

### 7.7 External Dependencies

| Tool            | Purpose                       |
| --------------- | ----------------------------- |
| Podman          | Container management          |
| Podman Compose  | Multi-container orchestration |
| Caddy           | Reverse proxy + auto TLS      |
| systemd/launchd | Service integration           |
