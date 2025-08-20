# Cadman — System Context

> **Audience:** PMs, ops, and engineers

> **Purpose:** Show how Cadman fits into the bigger picture — what’s inside, what’s outside, and how it interacts with users and external systems.

---

## 1. Overview

Cadman is a **command-line tool** that sits between **human operators / CI/CD pipelines** and **container/proxy runtimes** (Podman, Podman Compose, Caddy).

It **does not** replace those runtimes; instead, it provides:

* A **unified interface** to run them securely and consistently.
* **Configuration management** for apps and reverse proxies.
* **Automation** for port allocation, TLS setup, and service lifecycle.

---

## 2. External Actors

| Actor                    | Type            | Role                                                            |
| ------------------------ | --------------- | --------------------------------------------------------------- |
| **Developer / Sysadmin** | Human           | Uses Cadman CLI to manage projects, apps, and runtime services. |
| **CI/CD Pipeline**       | System          | Automates lifecycle commands for deployments.                   |
| **Podman CLI**           | External Tool   | Executes container lifecycle commands.                          |
| **Podman Compose**       | External Tool   | Orchestrates multi-container apps.                              |
| **Caddy**                | External Tool   | Acts as reverse proxy, handles TLS, routes traffic.             |
| **Operating System**     | Environment     | Provides filesystem, networking, process management.            |
| **Systemd / launchd**    | Service Manager | Controls background Cadman services if daemon mode enabled.     |

---

## 3. Context Diagram

```
            +---------------------------------------+
            |            Developer / Sysadmin       |
            |          (Human Operator)             |
            +---------------------------------------+
                           |  CLI commands
                           v
+-----------------------------------------------------------+
|                        Cadman CLI                         |
|-----------------------------------------------------------|
| - Command Router      - RunMode Resolver                  |
| - Config Loader       - Registry Manager                  |
| - Port Allocator      - Caddy Config Generator            |
| - Process Runner      - State Manager                     |
| - Log Aggregator                                         |
+-----------+------------------+---------------------------+
            |                  |                           
            v                  v                           
  +----------------+   +-----------------+                 
  | Podman CLI     |   | Podman Compose  |                 
  | (containers)   |   | (multi-service) |                 
  +----------------+   +-----------------+                 
            |                  |                           
            v                  v                           
         Containers      Multi-container stacks            
                                                          
            ^                                              
            |                                              
   +----------------+                                      
   |     Caddy      | <---- Reverse proxy + TLS -----------+
   +----------------+                                      
            |                                              
            v                                              
      Client Requests (HTTP/HTTPS)                         
```

---

## 4. Boundaries

### 4.1 Inside Cadman (control, own code)

* **CLI Command Parser** — dispatches subcommands.
* **RunMode Enforcement** — decides privilege level for subprocesses.
* **Configuration Management** — loads/validates `cadman.yaml`, `config.toml`, `apps.toml`.
* **Port Management** — allocates and reserves ports without conflicts.
* **App Registry** — tracks apps and their state.
* **Caddy Config Builder** — generates proxy/TLS configs.
* **State Tracking** — records runtime info (`state.toml`).
* **Logging** — aggregates and outputs logs in chosen format.

### 4.2 Outside Cadman (integrations, dependencies)

* **Podman** — must be installed; Cadman shells out to it.
* **Podman Compose** — must be installed; Cadman shells out to it.
* **Caddy** — must be installed; Cadman shells out or uses API.
* **Systemd / launchd** — for background service management.
* **OS Kernel & Networking** — process spawning, socket allocation, filesystem.

---

## 5. Communication Flows

### 5.1 Human-Initiated Flow

1. **User runs CLI command** (`cadman up`, `cadman app add`, etc.).
2. Cadman **loads configuration**, resolves RunMode, locks necessary resources.
3. Cadman **calls Podman/Compose/Caddy** as subprocesses or via API.
4. External tool runs, returns output and status code.
5. Cadman **updates registry/state** and returns results to user in chosen format.

### 5.2 CI/CD-Initiated Flow

1. CI/CD job triggers Cadman with non-interactive flags.
2. Cadman executes same flow as human-run command, but output is **machine-readable** (JSON/YAML).
3. Failures return **non-zero exit codes** and machine-readable error data for pipeline parsing.

---

## 6. Security Context (High-Level)

> Detailed threat model is in the **Security Model** doc.

* **Least privilege**: default RunMode is `cadman` (global) or `user` (local).
* **Explicit escalation only**: no silent privilege changes.
* **Safe binds**: defaults to `127.0.0.1` unless user config allows public exposure.
* **File locking**: prevents concurrent writes to registry and ports.
* **Rollback**: Caddy reload failures revert to last known good config.

---

## 7. Deployment Views

### 7.1 Local Dev

* Installed in `$HOME/.cargo/bin` or `/usr/local/bin`.
* User config: `$HOME/.config/cadman/config.toml`.
* Registry: `$HOME/.cadman/apps.toml`.
* RunMode: `user`.

### 7.2 Production Server

* Installed via package manager (`brew`, `apt`, `dnf` in future).
* Config: `/etc/cadman/config.toml`.
* Registry: `/var/lib/cadman/apps.toml`.
* Logs: `/var/log/cadman/`.
* Optional `cadman` system user/group.
* RunMode: `cadman`.

---

## 8. Key Context Decisions

* Cadman does **not** directly manage containers; Podman/Compose do.
* Cadman **does not persist secrets** beyond what’s needed in runtime configs (and redacts in logs).
* All **project config is user-editable**, but registry/state are managed by Cadman only.
* **APIs for Podman/Caddy may be adopted** in future for improved performance and reduced shelling-out.
