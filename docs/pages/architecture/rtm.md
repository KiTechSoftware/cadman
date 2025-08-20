# Cadman — Requirements Traceability Matrix (RTM)

> **Purpose:** Map every requirement in the SRS to implementation touchpoints and test coverage.

> **Scope:** Cadman v1 (Podman, Podman Compose, Caddy).

> **Audience:** engineers, QA, PMs.

**Legend**

* **Artefacts (R/W):** `cadman.yaml` (R), `config.toml` (R/W), `apps.toml` (R/W), `state.toml` (R/W), logs (W)
* **Tests:** U = Unit, I = Integration, E = End-to-End, S = Static (lint/fuzz), P = Performance
* **Exit Codes:** 0 OK, 1 usage, 2 not found, 3 dependency/perms, 4 validation, 5–9 internal, 10–19 external tool, 20–29 concurrency/timeouts

---

## 1) Core (FR-C)

| Req ID | Summary                        | Primary Commands                | Data (R/W)                                                   | External             | Exit Codes        | Tests   |
| ------ | ------------------------------ | ------------------------------- | ------------------------------------------------------------ | -------------------- | ----------------- | ------- |
| FR-C1  | Install Cadman                 | `install`                       | `config.toml` (W), logs (W)                                  | OS, fs               | 0,3,4,5           | U,I,E   |
| FR-C2  | Update Cadman                  | `update`                        | binary (W atomically), logs (W)                              | net/fs (channel), OS | 0,3,5             | U,I,E   |
| FR-C3  | Uninstall                      | `uninstall`                     | remove config/logs (opt), logs(W)                            | OS, fs               | 0,3,5             | I,E     |
| FR-C4  | Doctor 🩺                      | `doctor`                        | `config.toml`(R), `apps.toml`(R), logs(W)                    | Podman/Compose/Caddy | 0,3,4,5           | U,I,E   |
| FR-C5  | Configure ⚙️                   | `configure --get/--set/--reset` | `config.toml`(R/W)                                           | fs                   | 0,4,5             | U,I,E   |
| FR-C6  | Lifecycle (init/up/down/purge) | `init`,`up`,`down`,`purge`      | `cadman.yaml`(R/W for init), `state.toml`(W), `apps.toml`(R) | Podman/Compose/Caddy | 0,3,4,10–19,20–22 | U,I,E,P |
| FR-C7  | Port Allocation                | via `up`,`ports`                | `apps.toml`(R), `state.toml`(R/W)                            | OS sockets           | 0,4,21,22         | U,I,E,P |
| FR-C8  | Caddy Config                   | via `up`,`daemon reload`        | snapshot optional, logs(W)                                   | Caddy API/CLI        | 0,10–19           | I,E     |
| FR-C9  | Output/Verbosity               | global flags                    | n/a                                                          | n/a                  | n/a               | U,I,E,S |

---

## 2) Daemon (FR-D)

| Req ID | Summary          | Primary Commands | Data (R/W)                                        | External        | Exit Codes | Tests |
| ------ | ---------------- | ---------------- | ------------------------------------------------- | --------------- | ---------- | ----- |
| FR-D1  | daemon start ▶️  | `daemon start`   | `config.toml`(R), `apps.toml`(R), logs(W)         | systemd/launchd | 0,3,5      | I,E   |
| FR-D2  | daemon stop ⏹️   | `daemon stop`    | logs(W)                                           | systemd/launchd | 0,5        | I,E   |
| FR-D3  | daemon reload 🔁 | `daemon reload`  | `apps.toml`(R/W mark not\_found), `state.toml`(W) | Caddy           | 0,10–19    | I,E   |
| FR-D4  | daemon status 📊 | `daemon status`  | `state.toml`(R)                                   | systemd/launchd | 0,5        | I,E   |

---

## 3) Project (FR-P)

| Req ID | Summary              | Primary Commands    | Data (R/W)                        | External             | Exit Codes        | Tests |
| ------ | -------------------- | ------------------- | --------------------------------- | -------------------- | ----------------- | ----- |
| FR-P1  | Project config rules | applies to all      | `cadman.yaml`(R)                  | n/a                  | 0,4               | U,S,I |
| FR-P2  | Init ✨ (idempotent)  | `init`              | `cadman.yaml`(W), logs(W)         | fs                   | 0,4,5             | U,I,E |
| FR-P3  | Up/Down/Purge        | `up`,`down`,`purge` | `state.toml`(R/W), `apps.toml`(R) | Podman/Compose/Caddy | 0,3,4,10–19,20–22 | I,E,P |

---

## 4) App Registry (FR-A)

| Req ID | Summary       | Primary Commands | Data (R/W)                       | External     | Exit Codes | Tests |
| ------ | ------------- | ---------------- | -------------------------------- | ------------ | ---------- | ----- |
| FR-A1  | app add ➕     | `app add`        | `apps.toml`(W), `cadman.yaml`(R) | fs           | 0,2,4,20   | U,I,E |
| FR-A2  | app remove ➖  | `app remove`     | `apps.toml`(W)                   | fs           | 0,2,5,20   | U,I,E |
| FR-A3  | app ls 📜     | `app ls`         | `apps.toml`(R)                   | n/a          | 0          | U,I,E |
| FR-A4  | app info 🧠   | `app info`       | `apps.toml`(R), `cadman.yaml`(R) | n/a          | 0,2,4      | U,I,E |
| FR-A5  | app status 🩺 | `app status`     | `state.toml`(R), `apps.toml`(R)  | Podman/Caddy | 0,2,10–19  | I,E   |
| FR-A6  | app ports 🔎  | `app ports`      | `apps.toml`(R), `state.toml`(R)  | OS sockets   | 0,21       | U,I,E |
| FR-A7  | app logs 📓   | `app logs`       | logs(R)                          | Podman/Caddy | 0,10–19    | I,E   |

---

## 5) Wrappers (FR-W)

| Req ID | Summary    | Primary Commands | Data (R/W) | External       | Exit Codes | Tests |
| ------ | ---------- | ---------------- | ---------- | -------------- | ---------- | ----- |
| FR-W1  | Podman 🧰  | `pod`            | logs(W)    | Podman         | 0,10–19    | I,E   |
| FR-W2  | Compose 🧩 | `compose`        | logs(W)    | Podman Compose | 0,10–19    | I,E   |
| FR-W3  | Caddy 🪄   | `caddy`          | logs(W)    | Caddy          | 0,10–19    | I,E   |

---

## 6) Self-Introspection (FR-S)

| Req ID | Summary  | Primary Commands | Data (R/W)                      | External   | Exit Codes | Tests |
| ------ | -------- | ---------------- | ------------------------------- | ---------- | ---------- | ----- |
| FR-S1  | info 🧾  | `info`           | `config.toml`(R), env, flags    | n/a        | 0          | U,I   |
| FR-S2  | logs 🗂️ | `logs`           | logs(R)                         | n/a        | 0          | I,E   |
| FR-S3  | ports 🧮 | `ports`          | `apps.toml`(R), `state.toml`(R) | OS sockets | 0,21       | U,I,E |

---

## 7) Non-Functional Requirements (NFR)

| NFR ID | Theme                                      | Implementation Link                                  | Verification          |
| ------ | ------------------------------------------ | ---------------------------------------------------- | --------------------- |
| NFR-1  | Security (RunMode, no implicit escalation) | RunMode resolver, `--root` banner, systemd hardening | I,E, S (policy tests) |
| NFR-2  | Formats (table/json/yaml)                  | Global output layer                                  | U,I,E                 |
| NFR-3  | Performance (<500ms, ≤1s start)            | Optimized IO, cached scans                           | P (bench), I          |
| NFR-4  | Registry safety (atomic, lock)             | Lock files + atomic writes                           | I,E (fault injection) |
| NFR-5  | Logging defaults & rotation                | File logger + reopen on rotate                       | I,E                   |
| NFR-6  | Port mgmt (no duplicates)                  | OS probe + registry                                  | I,E,P                 |
| NFR-7  | Precedence (CLI>ENV>cfg>defaults)          | Config resolver                                      | U,I                   |
| NFR-8  | No clobber                                 | `--force` gates writes                               | U,I,E                 |
| NFR-9  | Actionable errors                          | Error envelope + next steps                          | U,I,E                 |
| NFR-10 | Zero-downtime Caddy                        | Validate→apply→verify→rollback                       | I,E                   |
| NFR-11 | CI/CD friendly                             | Non-interactive, JSON logs                           | I,E                   |
| NFR-12 | Atomicity (fsync/rename)                   | IO layer contract                                    | I,E (crash sims)      |
| NFR-13 | Identity (ULID)                            | `app add`                                            | U,I                   |
| NFR-14 | Schema versioning                          | `"schema_version"` in JSON                           | U,S,I                 |

---

## 8) Test Plan Pointers (per Feature)

> High-level pointers you can turn into test cases.

### 8.1 Install/Update/Uninstall

* **U:** parse flags, config path resolution.
* **I:** install in user vs global; perms failure → code 3.
* **E:** update with atomic swap; verify old binary not executed mid-swap.

### 8.2 Init/Up/Down/Purge

* **U:** YAML validation (happy + edge cases).
* **I:** Up with missing Podman → code 3; Caddy invalid JSON → rollback.
* **E:** Two concurrent `up` → project lock; `down` during `up` → timeout → 22.

### 8.3 Port Allocation

* **U:** range parsing, collision detection.
* **I:** Pre-bind OS socket to simulate conflict.
* **E:** `--auto-retry` remaps once and updates Caddy; otherwise 21.

### 8.4 Registry

* **U:** ULID, duplicate names.
* **I:** Lock contention (two `app add`), atomic write survival across SIGKILL.
* **E:** `app ls/info/status` consistent with `apps.toml` + `state.toml`.

### 8.5 Wrappers

* **I:** Exit code mapping 10–19; stdout/stderr passthrough.
* **E:** RunMode switching; `--root` prints banner and logs audit entry.

### 8.6 Introspection & Logging

* **U:** output formatting, schema\_version.
* **I:** `info` reflects precedence; `logs --follow` tails multi-source.

### 8.7 Performance

* **P:** cold vs warm runs for `apps|ports|info`; `up` start latency.

---

## 9) Coverage Matrix (Req → Tests)

| Req      | U  | I  | E  | S  | P  |
| -------- | -- | -- | -- | -- | -- |
| FR-C1    | ✔︎ | ✔︎ | ✔︎ |    |    |
| FR-C2    | ✔︎ | ✔︎ | ✔︎ |    |    |
| FR-C3    |    | ✔︎ | ✔︎ |    |    |
| FR-C4    | ✔︎ | ✔︎ | ✔︎ |    |    |
| FR-C5    | ✔︎ | ✔︎ | ✔︎ |    |    |
| FR-C6    | ✔︎ | ✔︎ | ✔︎ |    | ✔︎ |
| FR-C7    | ✔︎ | ✔︎ | ✔︎ |    | ✔︎ |
| FR-C8    |    | ✔︎ | ✔︎ |    |    |
| FR-C9    | ✔︎ | ✔︎ | ✔︎ | ✔︎ |    |
| FR-D1–D4 |    | ✔︎ | ✔︎ |    |    |
| FR-P1–P3 | ✔︎ | ✔︎ | ✔︎ |    |    |
| FR-A1–A7 | ✔︎ | ✔︎ | ✔︎ |    |    |
| FR-W1–W3 |    | ✔︎ | ✔︎ |    |    |
| FR-S1–S3 | ✔︎ | ✔︎ | ✔︎ |    |    |
| NFR-1    |    | ✔︎ | ✔︎ | ✔︎ |    |
| NFR-2    | ✔︎ | ✔︎ | ✔︎ |    |    |
| NFR-3    |    |    |    |    | ✔︎ |
| NFR-4    |    | ✔︎ | ✔︎ |    |    |
| NFR-5    |    | ✔︎ | ✔︎ |    |    |
| NFR-6    | ✔︎ | ✔︎ | ✔︎ |    | ✔︎ |
| NFR-7    | ✔︎ | ✔︎ |    |    |    |
| NFR-8    | ✔︎ | ✔︎ | ✔︎ |    |    |
| NFR-9    | ✔︎ | ✔︎ | ✔︎ |    |    |
| NFR-10   |    | ✔︎ | ✔︎ |    |    |
| NFR-11   |    | ✔︎ | ✔︎ |    |    |
| NFR-12   |    | ✔︎ | ✔︎ |    |    |
| NFR-13   | ✔︎ | ✔︎ |    |    |    |
| NFR-14   | ✔︎ | ✔︎ |    | ✔︎ |    |

*(Adjust ✔︎ as implementation progresses.)*

---

## 10) Artefact Trace (Req → Data)

| Req      | cadman.yaml | config.toml | apps.toml | state.toml | Logs |
| -------- | ----------- | ----------- | --------- | ---------- | ---- |
| FR-C1    |             | W           |           |            | W    |
| FR-C2    |             |             |           |            | W    |
| FR-C3    |             | W?          |           |            | W    |
| FR-C4    | R           | R           | R         | R          | W    |
| FR-C5    |             | W           |           |            | W    |
| FR-C6    | R/W\*       | R           | R         | W          | W    |
| FR-C7    | R           | R           | R         | R/W        | W    |
| FR-C8    | R           | R           | R         | W          | W    |
| FR-D1–D4 |             | R           | R/W       | R/W        | W    |
| FR-P1–P3 | R/W         |             |           | R/W        | W    |
| FR-A1–A7 | R           |             | R/W       | R/W        | W    |
| FR-W1–W3 |             | R           |           |            | W    |
| FR-S1–S3 | R           | R           | R         | R          |      |

\* `init` may write `cadman.yaml` idempotently.

---

## 11) Change Control & Versioning

* **SRS/HLD diffs** trigger RTM updates in the same PR.
* **Schema changes:** bump `schema_version` (JSON).
* **Exit code changes:** must be listed in this RTM and SRS §Exit Codes.
* **New commands/flags:** add FR row + tests before merging.

---

## 12) Open RTM Items

* Add concrete test IDs once test suite files exist (e.g., `tests/e2e/test_up_down.rs::e2e_up_down_basic`).
* Decide on **performance budgets** per target (macOS/Linux) and store baselines.
* Confirm **minimum supported Rust** version; add to verification checks.
