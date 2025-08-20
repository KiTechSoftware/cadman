# Cadman — Security Model

> **Audience:** engineers, ops, PMs

> **Goal:** define how Cadman minimises risk in design, configuration, and operations.

---

## 1) Security Objectives

* **Least privilege by default** — never escalate silently.
* **Predictable surfaces** — explicit, auditable integration with Podman/Compose/Caddy.
* **Safe concurrency** — race-free writes; no partial or torn state.
* **Zero trust on local state** — verify reality (sockets/processes) before acting.
* **Secure-by-default networking** — localhost binds unless the user opts into exposure.
* **Recoverable failures** — Caddy rollback, atomic updates, idempotent commands.
* **No secrets leakage** — redact in logs and outputs.

---

## 2) Trust Boundaries & Assets


```
+-----------------------+       +----------------------+       +--------------------+
|   Human / CI Runner   |  -->  |      Cadman CLI      |  -->  | Podman/Compose     |
|  (untrusted inputs)   |       | (trusted code path)  |       | Caddy (external)   |
+-----------------------+       +----------+-----------+       +---------+----------+
                                              |                          |
                                              v                          v
                                       Files & State              Network Sockets
```

**Assets to protect**

* Project configs (`cadman.yaml`)
* App registry (`apps.toml`)
* Runtime state (`state.toml`)
* Logs (Cadman + aggregated)
* Caddy live config
* Podman namespaces / containers (incl. volumes/ports)

---

## 3) Threat Model (STRIDE snapshot)

| Category               | Example Threat                              | Mitigation (v1)                                           |
| ---------------------- | ------------------------------------------- | --------------------------------------------------------- |
| Spoofing               | Unauthorised process controlling Cadman ops | RunMode + per-project lock + explicit escalation          |
| Tampering              | Concurrent writes corrupt `apps.toml`       | Advisory lock + atomic write + fsync + rename             |
| Repudiation            | “Who changed ports?”                        | Structured logs with UTC timestamps & command lines       |
| Information Disclosure | Secrets in logs                             | Redaction; never echo env values; mask well-known keys    |
| DoS                    | Port allocator starvation / long lock holds | Backoff + timeouts + exit codes 20–29; guidance in errors |
| Elevation of Privilege | Silent root escalation                      | Explicit `--root` only; loud banner & audit log entry     |

---

## 4) Attack Surfaces & Controls

1. **CLI Inputs**

* Validate flags/paths; reject control chars/`..` traversal.
* Disallow destructive defaults; require `--force` for overwrites.

2. **File System**

* Registry/state writes via **temp → fsync → rename → fsync(dir)**.
* Lock files: `apps.toml.lock`, `ports.lock`, `${project}/.cadman.lock`.
* Permissions (Linux):

  * `/var/lib/cadman`: `0750 cadman:cadman`
  * `/var/log/cadman`: `0750 cadman:cadman`
  * `/etc/cadman`: `0750 root:cadman`
  * `$HOME/.cadman`: `0700 $USER:$USER`

3. **Process Execution**

* Spawn `podman`/`podman-compose`/`caddy` with **explicit RunMode**.
* Preserve/return external exit codes (map to 10–19).
* Timeouts on long-running child processes; kill tree on cancel.

4. **Networking**

* Default container binds & Caddy upstreams **127.0.0.1** only.
* Caddy **admin API limited to localhost** (or disabled if requested).
* No ephemeral public listeners without explicit YAML.

5. **Logging**

* Default to file logs; structured; UTC ISO-8601.
* Redact variables with names matching `*_KEY`, `*_SECRET`, `*_TOKEN`, `PASSWORD`, etc.
* Rotation via `rename()`; writers reopen; no truncation.

---

## 5) RunMode & Privilege Controls

### Policy

* **Global install** → default `RunMode=cadman` (dedicated user/group).
* **User install** → default `RunMode=user`.
* **Root** → **only** with `--root` and a warning banner, recorded in logs.

### Systemd unit (global)

```ini
[Service]
User=cadman
Group=cadman
NoNewPrivileges=true
ProtectSystem=full
ProtectHome=true
PrivateTmp=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
RestrictSUIDSGID=true
RestrictNamespaces=true
RestrictRealtime=true
SystemCallFilter=@system-service
LockPersonality=true
AmbientCapabilities=
CapabilityBoundingSet=
```

> macOS (launchd): run as non-admin user; avoid `sudo` for child processes unless `--root`.

---

## 6) Secrets Handling

* **Do not store** secrets in Cadman state/registry.
* Support `env_file`/ENV for Podman/Compose; **never echo values**.
* Redact known secret keys in `info/status/logs`.
* If users insist on YAML secrets (not recommended), warn and mask at display.

---

## 7) Networking & TLS (Caddy)

* Caddy config **pre-validated**; reload atomically; rollback on failure.
* Admin endpoint bound to `127.0.0.1` by default (or disabled).
* Strong recommendation: TLS termination in Caddy, upstreams on loopback.

**Hardening snippet (Caddy admin):**

```json
{
  "admin": { "listen": "127.0.0.1:2019" }
}
```

---

## 8) Concurrency & Race Avoidance (security-relevant)

* **Single writer** locks for `apps.toml`, `state.toml`, port allocator.
* **Mid-lifecycle port conflicts:**

  * Default: **fail fast** (exit 21), report remediation.
  * Optional `--auto-retry`: try one alternative port → update Caddy → emit remap in output.
* **Per-project operations:** `.cadman.lock`; timeout → exit 22 (guidance shown).

---

## 9) Supply Chain & Build Integrity

* Pin Rust toolchain (`rust-toolchain.toml`) and MSRV.
* Reproducible builds: `--locked`, no network during build steps beyond crates.
* Vendor or checksum **Cargo.lock** in repo; disallow wildcard versions.
* Release artefacts: checksum (SHA-256) + provenance (SBOM if you want).
* Updates: channel-aware, atomic binary replace; verify checksum/signature.

---

## 10) CI/CD Hardening

* Run jobs as **non-root**; isolate workspace.
* Use **OIDC** (workload identity) to cloud registries, not long-lived tokens.
* Require JSON output for parsers; fail on non-zero exit codes.
* Enforce branch protections and mandatory checks for `main`.

---

## 11) Configuration Defaults (secure)

* **Binds:** loopback only unless specified.
* **RunMode:** `cadman` (global) / `user` (local).
* **Caddy admin:** `127.0.0.1` or disabled.
* **Logs:** file, minimal verbosity.
* **No clobber:** refuse overwrites without `--force`.
* **Compose lookup:** prefer `podman-compose.yaml`; **ignore Docker files** by default.

---

## 12) Incident Response & Auditability

* **What to capture (always):** command line, user/runmode, target path/app\_id, exit code, timing.
* **Where:** `/var/log/cadman/*.log` (global) or `$HOME/.cadman/logs`.
* **How to roll back:** re-apply last known good Caddy config (kept as snapshot on disk).
* **How to recover registry:** last write wins; atomic rename ensures either old or new file exists.

---

## 13) Security Testing (what to automate)

* **Static checks:** deny secrets, lint for dangerous syscalls/`Command` use.
* **Unit/integration:** port allocator races; registry atomicity; rollback paths.
* **Smoke:** `init→up→down→purge` under simultaneous invocations.
* **Fuzz:** YAML loader & env substitution parsing.
* **e2e:** Caddy validation/reload; confirm rollback on broken config.

---

## 14) Platform Hardening

### Linux (recommended)

* Create dedicated user/group `cadman`; add operators to group.
* Set directory perms as above; use `umask 027`.
* SELinux/AppArmor: prefer **Podman default confinement**; do not add `--privileged`.
* If binding <1024, use Caddy for privileged ports; keep apps on high ports.

### macOS

* Run under a standard user; no `sudo` for normal ops.
* Logs under `~/Library/Logs/cadman` (or `$HOME/.cadman/logs`).
* Launchd jobs: avoid `KeepAlive` unless running daemon; set `LowPriorityIO`.

---

## 15) Operational Checklists

### Install (global)

* [ ] Create `cadman` user/group.
* [ ] `/etc/cadman` `0750 root:cadman`
* [ ] `/var/lib/cadman` `0750 cadman:cadman`
* [ ] `/var/log/cadman` `0750 cadman:cadman`
* [ ] Caddy admin bound to `127.0.0.1`.

### Project Onboarding

* [ ] `cadman.yaml` binds on `127.0.0.1` unless public exposure is intended.
* [ ] No secrets embedded in YAML; use `env_file`.
* [ ] `cadman app add` executed; verify ULID assigned.
* [ ] `cadman up` → validate Caddy → reload → confirm routes OK.

### Production Run

* [ ] `cadman daemon start` under `cadman` user.
* [ ] Logs shipping configured (tail to journald/syslog or agent).
* [ ] Backups: `/etc/cadman`, `/var/lib/cadman`.

### Incident

* [ ] Collect Cadman logs + Caddy logs.
* [ ] Roll back Caddy to last known good.
* [ ] Verify port allocator vs OS sockets; release/reaquire as needed.

---

## 16) Open Security Items (track in issues)

* Signed releases & SBOM publication.
* Optional policy to **forbid Docker compose** entirely (compile-time feature).
* Pluggable secret redaction dictionary (per env).
* Optional sqlite registry to strengthen concurrent write patterns at scale.

---

## 17) Appendix: RunMode Banner (root)

When `--root` is used, print once per invocation:

```
⚠️  Elevated privileges requested (--root). Actions will run as root.
    This is logged and may affect system-wide resources.
```
