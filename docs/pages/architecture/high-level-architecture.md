# Cadman — High-Level Design (HLD)

> **Audience:** engineers, PMs, ops

> **Purpose:** Define the high-level architecture for Cadman v1, including major components and their interactions.

## 1. Goals & Non-Goals

### 1.1 Goals

* Provide a single, least-privilege CLI to manage Podman/Compose and Caddy across Linux/macOS.
* Keep configuration predictable (YAML project config; TOML global/registry/state).
* Ensure safe concurrency: atomic writes, advisory locks, idempotent commands.
* Offer zero-downtime Caddy reloads and deterministic port allocation.
* Be CI/CD friendly with machine-readable output.

### 1.2 Non-Goals

* No Docker or Docker Compose in v1.
* No built-in DNS providers or certificate issuance APIs (Caddy handles TLS).
* No always-on Cadman daemon (optional systemd/launchd unit only).
* No GUI or web UI.

---

## 2. Architecture Overview

### 2.1 Component Map

```
+-------------------------------------------------------------+
|                         Cadman CLI                          |
|  (Rust binary)                                              |
|  - Command Router                                           |
|  - RunMode Resolver                                         |
|  - Config Loader/Resolver                                   |
|  - Registry Manager (apps.toml)                             |
|  - Port Allocator                                           |
|  - Caddy Config Generator (JSON/Caddyfile)                  |
|  - Process Runner (podman, podman-compose, caddy)          |
|  - State Manager (state.toml)                               |
|  - Log Aggregator                                           |
+--------------------+-------------------+--------------------+
                     |                   |
                     v                   v
            +----------------+    +----------------+
            |  Podman CLI    |    | Podman Compose|
            |  (containers)  |    | (multi-svc)   |
            +----------------+    +----------------+
                     |
                     v
                 Containers
                     ^
                     |
            +----------------+
            |    Caddy       |
            | (reverse proxy)|
            +----------------+
```

### 2.2 Runtime Modes (RunMode)

* `cadman` (default for global installs): run as dedicated `cadman` user/group.
* `user` (default for user installs): run as invoking user.
* `root`: explicit only (`--root`).

---

## 3. Deployment Topologies

### 3.1 User Install (Dev Laptop)

* Paths under `$HOME`:

  * Config: `$HOME/.config/cadman/config.toml`
  * Registry/State: `$HOME/.cadman/`
  * Logs: `$HOME/.cadman/logs/`
* RunMode: `user` by default.

### 3.2 Global Install (Server)

* Paths under system locations:

  * Config: `/etc/cadman/config.toml`
  * Registry/State: `/var/lib/cadman/`
  * Logs: `/var/log/cadman/`
* `cadman` user/group optionally created.
* RunMode: `cadman` by default.
* systemd/launchd unit optional for daemon commands.

---

## 4. Major Subsystems

### 4.1 Command Router 🧭

* Parses `cadman <subcommand> [flags]`.
* Dispatches to handlers with resolved **effective config** (CLI > ENV > config > defaults).
* Enforces output format and global verbosity flags.

### 4.2 RunMode Resolver 🧑‍⚖️

* Chooses `cadman|user|root` given install scope and flags.
* Implements **no implicit escalation**; error if privileged op required without `--root`.

### 4.3 Config Loader/Resolver ⚙️

* Loads:

  * Global config (`config.toml`)
  * Project config (`cadman.yaml|yml`)
  * Registry (`apps.toml`)
  * State (`state.toml`)
* Performs env-substitution and schema validation.
* Emits a **resolved view** for downstream components.

### 4.4 Registry Manager (apps.toml) 📚

* ULID app IDs, name, path, status, last\_seen.
* Single-writer with advisory file lock; atomic updates (temp → fsync → rename).
* Safe read concurrency.

### 4.5 Port Allocator 🔌

* Configurable ranges; prefers loopback binds.
* Checks **registry claims + OS sockets** before reserving.
* Writes reservations atomically; releases on `down` or explicit purge.
* Mid-lifecycle conflicts handled per §8.

### 4.6 Caddy Config Generator 🧩

* Builds **Caddy JSON** (default) or **Caddyfile** on request.
* Validates before apply; zero-downtime reload; rollback on failure.
* Redacts secrets in logs/outputs.

### 4.7 Process Runner 🚀

* Spawns `podman`, `podman-compose`, `caddy` using correct RunMode.
* Captures stdout/stderr, preserves exit codes, timeouts, and retries where applicable.

### 4.8 State Manager (state.toml) 🗂️

* Ephemeral runtime snapshots (container status, ports, route health).
* Atomic writes; tolerant readers.

### 4.9 Log Aggregator 📝

* Collates Cadman’s own logs with app/container/Caddy logs (tagged sources).
* Supports `--follow|--since|--tail|--level`.
* Rotation via rename; writers reopen.

---

## 5. Data & Files (Operational Footprint)

| File          | Scope       | Purpose                           | Concurrency                        |
| ------------- | ----------- | --------------------------------- | ---------------------------------- |
| `config.toml` | global/user | Global defaults & preferences     | R/O at runtime                     |
| `cadman.yaml` | per project | Declarative project config        | R/O; may be edited by user         |
| `apps.toml`   | per scope   | App registry (ULID, path, status) | Single-writer lock + atomic writes |
| `state.toml`  | per scope   | Runtime snapshot (ephemeral)      | Single-writer lock + atomic writes |
| logs dir      | per scope   | Cadman + app logs                 | Append-only; rotate via rename     |

> Full schemas and examples live in the SRS Appendices (cross-reference there in docs).

---

## 6. Key Flows (ASCII Sequence)

### 6.1 `cadman init` ✨

```
User → CLI → Config Loader (defaults+answers)
    → Check cwd (cadman.yaml?) → Create/merge YAML (idempotent)
    → Optionally derive service/Caddy snippets
    → Write files (no clobber unless --force)
    → Done
```

### 6.2 `cadman up` 🚀

```
User → CLI → Resolve RunMode → Load project YAML
    → Registry read (app exists?) → Port Allocator (reserve if needed)
    → Generate Caddy config (JSON/Caddyfile) → Validate
    → Process Runner: podman/podman-compose up
    → Apply Caddy (zero-downtime reload; rollback on failure)
    → Update state.toml (atomic)
    → Done (human+machine output)
```

### 6.3 `cadman down` ⏹️

```
User → CLI → Resolve RunMode → Load project YAML
    → Process Runner: compose/podman down
    → Optionally release ports
    → Update state.toml
    → Done
```

### 6.4 `cadman app add` ➕

```
User → CLI → Validate path + YAML
    → Registry lock (exclusive) → ULID → Write atomic
    → Done
```

### 6.5 `cadman daemon reload` 🔁

```
User/Unit → CLI → Config refresh
    → Re-scan registry (mark not_found)
    → Re-emit Caddy config (validate→apply)
    → Reconcile services
    → Done
```

---

## 7. Error Handling & Exit Codes

* **Classify** errors: usage (1), not found (2), dependency/perms (3), validation (4), internal (5–9), external process (10–19), concurrency/timeouts (20–29).
* **Always** return actionable messages (summary, cause if known, next step).
* **Preserve** external tool exit codes by mapping to 10–19 where needed.

---

## 8. Concurrency & Race Conditions 🔐

### 8.1 Principles

* **Single writer** per shared file (lockfile).
* **Atomic writes** (temp → fsync → rename → fsync dir).
* **Idempotency** for all lifecycle commands.
* **OS-level checks** (don’t trust registry alone).

### 8.2 Hotspots & Mitigations

* **Registry (`apps.toml`)**: exclusive lock on write; retries with exponential backoff → exit 20 if exhausted.
* **Port allocation**: exclusive allocator lock; check registry and sockets; reserve and persist atomically.
* **Mid-lifecycle port conflict**:

  * Detect (health check fails or bind error).
  * Default: **fail fast** with exit 21 and clear remediation.
  * If `--auto-retry`: try **one** alternative in allowed range, update Caddy, reload, record remap in output.
* **Compose up/down**: per-project lock `.cadman.lock`; timeout escalates to exit 22.
* **Caddy reload**: validate before apply; rollback on failure; no partial state writes.
* **Log rotation**: rename files; writers reopen (signal or periodic reopen).
* **RunMode escalation**: never escalate implicitly; if blocked by lower-priv lock, exit 23 with guidance.

---

## 9. Security Posture (Design Hooks) 🛡️

> Full details in **Security Model** doc; this section lists design anchors implemented by HLD.

* **Least privilege** defaults (`cadman` or `user`).
* **Explicit escalation** only (`--root`).
* **Secrets redaction** in logs and outputs.
* **Safe defaults** for binds (127.0.0.1 unless explicitly exposed).
* **Config no-clobber**; `--force` required for overwrites.
* **Binary update** is atomic and verified (channel-aware).

---

## 10. Observability & Diagnostics 📊

* **Logs:** structured, include source tags (`cadman|podman|compose|caddy`).
* **doctor 🩺:** dependency checks, config validation, suggested fixes, `--fix` attempts.
* **Status endpoints:** via CLI (`info`, `status`, `ports`) with `--output table|json|yaml`.
* **Timestamps:** normalized to UTC ISO-8601.

---

## 11. Performance Expectations ⚡

* Read-only commands (`apps|ports|info`) < **500ms** for ≤ 20 apps.
* Lifecycle commands start within **1s**.
* Port allocation O(n) in range + OS check, amortized by caching reserved sets in memory during command execution.

---

## 12. Scalability & Limits

* Target: small to medium estates (tens of apps per host).
* Registry: append-friendly TOML; consider sqlite in future if write contention grows.
* Caddy: JSON size grows with routes; segment per app if needed in future.

---

## 13. Configuration Strategy

### 13.1 Precedence

```
Defaults  <  Global config  <  Environment (CADMAN_*)  <  CLI flags
(Resolved once per command invocation)
```

### 13.2 Validation

* Strict schema validation for `cadman.yaml` before lifecycle commands.
* Warn on unknown keys; allow forward compatibility where safe.

---

## 14. External Integrations

* **Podman / Podman Compose**: shell out via Process Runner; capture exit codes and logs.
* **Caddy**: prefer JSON API; Caddyfile on `--caddyfile`. Validate → apply → verify.

---

## 15. Risks & Mitigations

| Risk                                  | Impact            | Mitigation                                                                |
| ------------------------------------- | ----------------- | ------------------------------------------------------------------------- |
| Compose vs Docker ambiguity           | Misconfiguration  | Clearly document Podman Compose scope; feature flag Docker support later. |
| Port allocator contention             | Start delays      | Lock + backoff + clear exit 20; consider persistent reservation index.    |
| Caddy reload failure                  | Downtime risk     | Validate first; rollback to last known good; surface clear error.         |
| Privilege misuse                      | Security incident | No implicit escalation; loud warnings if `--root`.                        |
| Registry corruption (crash mid-write) | Orphaned entries  | Atomic writes only; fsync; journal logs / recovery guidance.              |

---

## 16. Open Questions (to resolve before GA)

* Should we maintain a **port reservation ledger** per app in `apps.toml` to speed free-port queries?
* Do we allow **partial success** on multi-service `up`, or fail the whole operation?
* What’s the **minimum supported Rust** version and target triples for release builds?
* How do we expose **health probes** (periodic vs on-demand only)?

---

## 17. Roadmap Cues (from HLD → SRS alignment)

* v1.0: current scope; Podman + Caddy only, strong CLI, zero-downtime reloads, safe concurrency.
* v1.x: structured schema exports, richer `status` (per-container health), optional Docker Compose.
* Candidate v2.0: replace TOML registry with sqlite, optional gRPC/HTTP control plane.

---

## 18. Appendix: RunMode Decision (ASCII)

```
Global install?
  ├─ YES → user in 'cadman' group? → YES: RunMode=cadman
  │                                  NO:
  │     --user? → RunMode=user
  │     --root? → RunMode=root
  │     else    → RunMode=cadman
  └─ NO (user install):
        --root? → RunMode=root
        --user? → RunMode=user
        default → RunMode=user
```

## 19. Appendix: Lifecycle Workflow (ASCII)

```
init → up → (status/logs/ports/info) → down → purge
   \_______________________________________________/
                idempotent & safe to repeat
```

