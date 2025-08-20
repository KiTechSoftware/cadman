# Cadman — Executive Summary

> **SRS:** See the [SRS](srs.md) for the full contract.

> **RTM:** See the [RTM](rtm.md) for traceability.

## Purpose

Cadman is a **Rust-based command-line tool** that simplifies deployment and management of containerized applications using **Podman**, **Podman Compose**, and **Caddy**.
It enables **secure, least-privilege automation** of application lifecycles, reverse proxy setup, and TLS management across **Linux** and **macOS**.

## Why Cadman Exists

Development and operations teams often face **complex, fragmented workflows** when managing containers and reverse proxies.
Cadman addresses this by providing:

* **One CLI for all container and proxy tasks**.
* **Security by default** — least privilege execution, no silent privilege escalation.
* **Faster onboarding** with ready-to-use project templates.
* **Automated port and TLS management** to avoid manual errors.

## Target Users

* **Developers**: Start/stop dev environments easily.
* **DevOps Engineers**: Manage production services without heavy orchestration layers.
* **System Administrators**: Standardize secure app deployment.

## Key Capabilities

| Feature                                    | Benefit                                                |
| ------------------------------------------ | ------------------------------------------------------ |
| **Project Initialization (`cadman init`)** | Create ready-to-run configs in seconds.                |
| **App Registry**                           | Track/manage all Cadman-managed apps in one place.     |
| **Secure RunModes**                        | Least-privilege by default; explicit escalation.       |
| **Wrapper Commands**                       | Consistent Podman, Compose, and Caddy execution.       |
| **Port Allocation**                        | Auto conflict-free port assignment.                    |
| **Zero-Downtime Caddy Reload**             | Seamless TLS/proxy config updates.                     |
| **Daemon Mode**                            | Optional background service for continuous management. |
| **Cross-Platform Support**                 | Linux, macOS, partial WSL support.                     |

## How Cadman Works (High Level)

1. **Configuration** — Define containers, services, and proxy rules in `cadman.yaml`.
2. **Execution Context** — Choose `cadman`, `user`, or `root` mode for commands.
3. **Automation** — Cadman handles ports, generates Caddy configs, and interacts with Podman/Compose.
4. **Idempotence** — Commands can be re-run without duplicates, drift, or downtime.

---

## Visual Aid: Cadman Workflow

```
   +----------------+
   | cadman init    |  ➡  Creates cadman.yaml + optional service & Caddy configs
   +----------------+
           |
           v
   +----------------+
   | cadman up      |  ➡  Starts containers, allocates ports, applies Caddy config
   +----------------+
           |
           v
   +----------------+
   | cadman down    |  ➡  Stops containers, keeps reservations unless released
   +----------------+
           |
           v
   +----------------+
   | cadman purge   |  ➡  Removes containers/volumes, cleans registry (safe by default)
   +----------------+
```

---

## Visual Aid: RunMode Decision Cheat Sheet

```
RunMode Selection Logic
=======================

Global install? 
 ├─ YES → User in 'cadman' group?
 │       ├─ YES → RunMode=cadman (default)
 │       ├─ NO  → Explicit --user? → RunMode=user
 │                Explicit --root? → RunMode=root
 │                Else → RunMode=cadman
 │
 └─ NO (User install)
         ├─ Explicit --root? → RunMode=root
         ├─ Explicit --user? → RunMode=user
         └─ Default → RunMode=user
```

---

## Roadmap (Short-Term)

* **v1.0** — Core commands, port allocation, app registry, secure defaults, zero-downtime reloads.
* **v1.1–v1.2** — More output formats, schema tools, enhanced status reporting.
* **Future** — Optional Docker Compose support, richer Podman/Caddy API use.

## Value Proposition

Cadman offers:

* **Simplicity** — One CLI for containers + proxy.
* **Security** — Safe defaults, minimal privilege.
* **Portability** — Same workflow for local and production.
* **Consistency** — Unified UX across environments.
