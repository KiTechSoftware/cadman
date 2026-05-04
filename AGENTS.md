# Cadman AGENTS.md

## Product Identity

Cadman is a Linux-only, Podman-first, Caddy-first proxy orchestration and workflow platform written in Rust.

Cadman discovers and manages Podman workloads, generates Caddy site configuration, and provides a controlled CLI-first workflow for local and server environments.

Cadman is being rebuilt from scratch with a scoped, maintainable path to v1.

Cadman v1 does **not** support Docker. Docker support is deferred to v1+ and must not influence v1 architecture, command design, or implementation.

## Non-Goals for v1

Do not add support for:

```txt
Docker
Docker socket discovery
Docker Compose as a Docker runtime
Traefik integration
Windows
macOS
Kubernetes
distributed orchestration
generic plugin hosting
```

Users who want Docker-first proxy automation should use Traefik or another Docker-native tool.

---

# Architecture Boundary

Cadman uses a sovereign architecture with one primary dependency direction:

```txt
cli -> core -> engine
```

## Source Layout

```txt
workspace/
  src/
    main.rs
    lib.rs

    cli/        # Thin user-intent adapter
    core/       # Application workflows, orchestration, UI rendering
    engine/     # Sovereign Cadman capabilities, models, errors, config, state
```

## Dependency Rules

```txt
engine imports nothing from cli or core
core imports engine
cli imports core
cli may import engine error types for error mapping and presentation
```

Do not create top-level modules for:

```txt
daemon/
api/
webui/
ui/
infra/
```

Daemon, API, Web UI, Podman, Caddy, reconcile, and serve behavior belong under:

```txt
engine/capabilities/
```

---

# Layer Responsibilities

## `cli/`

The CLI is a thin user-intent adapter.

CLI may:

```txt
parse command-line arguments
perform CLI-only syntax validation
validate mutually exclusive flags
convert args into core call parameters or request structs
call core workflows
return exit codes
perform minimal top-level error handling
```

CLI must not:

```txt
call Podman directly
call Caddy directly
use veltrix Podman/Caddy clients
use std::process::Command
read or write Cadman state files
read or write registry.toml
read or write config.toml
scan project directories
parse cadman.toml or cadman.yaml
inspect containers
filter containers beyond CLI flag validation
compute effective RunMode
perform authorization checks
decide whether Caddy should reload
generate Caddy site files
compare hashes
perform reconciliation
start daemon/API/Web UI internals directly
contain business rules
add new Scriba rendering
render tables
format reports
print normal command output
```

CLI captures user intent only.

## `core/`

Core owns Cadman’s user-facing workflow experience.

Core may:

```txt
orchestrate use-cases
coordinate engine operations
apply command-level workflow rules
perform workflow-level validation
call engine capabilities
choose output shape
render reports using Scriba
print user-facing output
map structured reports into tables, JSON envelopes, logs, or text
coordinate runtime context
route authorization-aware workflows through engine policy
```

Core must not:

```txt
call Podman directly when engine capability functions exist
call Caddy directly when engine capability functions exist
bypass engine state/config/registry abstractions
perform low-level external tool parsing
own domain models that belong in engine
own Podman/Caddy integration details
```

Core turns CLI intent into a Cadman workflow.

## `engine/`

Engine is Cadman’s sovereign capability layer.

Engine owns:

```txt
domain models
typed errors
constants
config.toml handling
registry.toml handling
state.json handling
Podman capability
Caddy capability
daemon capability
API capability
Web UI capability
reconciliation capability
process/system primitives
business rules
runtime policy
authorization primitives
RunMode behavior
```

Engine must not:

```txt
import cli
import core
render user-facing UI
print normal command output
depend on terminal formatting
own command-line argument parsing
```

Engine performs Cadman capability work and returns typed data/results.

---

# One-Line Architecture Rule

```txt
CLI captures intent, core renders experience, engine performs capability work.
```

---

# Engine Capabilities Layout

Use this structure:

```txt
engine/
  capabilities/
    podman/        # Podman discovery, wrapper behavior, container metadata
    caddy/         # Caddy config files, validation, reload, read-only API access
    reconcile/     # Desired-vs-actual comparison and apply planning
    daemon/        # Background live reconciler capability
    api/           # API runtime capability
    webui/         # Web UI runtime capability
    serve/         # Runtime serving capability if distinct

  models/          # App, Project, Route, Container, Runtime, RunMode
  errors/          # ErrorKind, ErrorCode, Error, Result<T>
  config/          # config.toml handling
  registry/        # registry.toml handling
  state/           # state.json handling
  constants/       # labels, paths, env vars, defaults
  system/          # filesystem/process primitives only
```

Low-level reusable primitives may live under:

```txt
engine/system/
```

Allowed examples:

```txt
filesystem helpers
path helpers
process abstraction
atomic file writes
locks
terminal-size primitives only if not presentation-specific
```

Do not recreate the old `infra` architecture.

Avoid:

```txt
engine/system/infra/
```

Prefer:

```txt
engine/capabilities/podman/
engine/capabilities/caddy/
engine/system/process/
engine/system/fs/
```

---

# Scriba Boundary

Core owns Cadman UI rendering.

Scriba may be used in:

```txt
core -> scriba    allowed and preferred for Cadman command output
cli  -> scriba    do not add new usage; existing minimal usage may remain
engine -> scriba  avoid for rendering; acceptable only for shared runtime output config types already in use
```

Use Scriba for:

```txt
tables
JSON/text/markdown output
output envelopes
logging
prompts
color policy
verbosity/quiet behavior
```

Rules:

```txt
Use Scriba instead of hand-rolled terminal rendering.
Use Scriba instead of ad hoc table formatting.
Use Scriba instead of custom prompt helpers.
Do not add new Scriba rendering in cli.
Do not render terminal output in engine.
Core may render structured engine reports using Scriba.
```

Preferred pattern:

```txt
cli parses intent
core calls engine
engine returns model/report
core renders model/report using Scriba
```

Example:

```txt
cli/cmd/containers.rs
  parses --all, --running, --stopped, --labels
  validates CLI-only flag conflicts
  calls core::usecases::containers::list(...)

core/usecases/containers.rs
  receives user intent
  calls engine::capabilities::podman::list_containers(...)
  renders ContainerListReport using Scriba

engine/capabilities/podman/
  talks to Podman using Veltrix
  returns ContainerListReport
```

---

# Veltrix Boundary

Use Veltrix wherever Cadman interacts with Linux/system/process/Podman/Caddy primitives.

Veltrix may be used in:

```txt
engine -> veltrix  allowed and preferred
core -> veltrix    avoid
cli -> veltrix     avoid
```

Use Veltrix for:

```txt
Podman CLI access
Podman socket access
Caddy CLI/API access where available
process execution
UID/GID/user/group lookup
system/user path resolution
Linux runtime helpers
Unicode/constants already provided by Veltrix
```

Rules:

```txt
Use Veltrix instead of raw std::process where practical.
Use Veltrix instead of manually shelling out to podman/caddy when typed clients exist.
Use Veltrix instead of custom UID/GID lookup logic where possible.
Use Veltrix path helpers instead of hard-coded Linux paths where possible.
Do not duplicate functionality already provided by Veltrix.
Do not use veltrix::services::podman::* in cli.
Do not use std::process::Command in cli.
```

Preferred pattern:

```txt
cli captures command intent
core chooses workflow
engine capability uses Veltrix
engine returns typed Cadman model/result
core renders user-facing output
```

---

# Runtime and RunMode

Cadman supports scoped RunMode behavior:

```txt
current
cadman
root
```

Root mode does not mean unrestricted root authority.

```txt
cadman --mode root = root-scope Cadman authority, not arbitrary root authority
```

Root mode may allow Cadman to:

```txt
inspect root-level Podman container metadata
manage Cadman-owned root-level state paths
write Cadman-owned Caddy site files
validate Caddy configuration
reload root-level Caddy after successful validation
```

Root mode must not allow Cadman to:

```txt
execute arbitrary root shell commands
run arbitrary podman commands as root
mutate non-Cadman Caddy configuration
start/stop/delete/modify root-owned containers unless explicitly scoped
bypass Cadman policy checks
silently escalate privileges
```

Regular users must not be allowed to enter root mode directly.

For v1:

```txt
current -> current user scope
cadman  -> cadman user/group policy
root    -> effective UID 0 or explicitly scoped authorization only
```

Do not silently downgrade or escalate without a clear policy decision.

---

# Optional `cadman-root` Group

A `cadman-root` group may exist as a limited delegated operator group.

Membership in `cadman-root` does not grant full root mode.

Allowed v1 scope:

```txt
read rootful Podman container metadata
read root-level Caddy state/config for drift detection
write Cadman-owned Caddy site files under a configured Cadman-managed directory
validate Caddy config
reload Caddy after successful validation
```

Not allowed:

```txt
arbitrary root command execution
arbitrary root Podman commands
starting/stopping/deleting root-owned containers by default
writing non-Cadman Caddy files
unrestricted root mode
```

---

# Daemon, API, and Web UI

Daemon, API, and Web UI are engine capabilities, not top-level layers.

Users interact with them through CLI commands.

```txt
user -> cli -> core -> engine/capabilities/*
```

## Daemon

The Cadman daemon is a background live reconciler.

It watches:

```txt
managed projects
registry state
Podman runtime state
Caddy site configuration
```

It applies safe updates only when the effective desired state changes.

Cadman is not the gateway router. Caddy is.

```txt
Cadman daemon = background service + live reconciler
Caddy = gateway router
CLI = standalone control surface or daemon client
```

The CLI must work when no daemon is running. When the daemon is running, the CLI may act as a client for daemon-backed operations.

This supports:

```txt
cadman serve          # non-systemd or containerized runtime
cadman serve --daemon # systemd-managed service operation
```

## API

The API is an engine capability that the CLI can start, stop, configure, or inspect.

The API is not the primary v1 user-facing surface.

## Web UI

The Web UI is an engine capability that the CLI can start, stop, configure, or inspect.

When Web UI is running, it should use the best available Cadman data source:

```txt
1. API, when running and reachable
2. daemon state, when daemon is running and current
3. direct Cadman files, when neither API nor daemon is available
```

The Web UI must not bypass registry, state, config, authorization, or RunMode rules.

---

# State and Config Source of Truth

Cadman state/config files are authoritative for Cadman-managed apps and behavior.

Use:

```txt
registry.toml  # managed apps/projects, locations, desired status
state.json     # observed runtime state: running, stopped, missing, changed
config.toml    # Cadman settings
```

Caddy site files are generated artifacts, not the source of truth.

Users may maintain external Caddy configuration, but Cadman-owned Caddy site files must not be manually edited because Cadman may overwrite or remove them during reconciliation.

---

# Caddy Rules

Cadman supports:

```txt
managed Caddy server
external/user-managed Caddy instance
```

Cadman generates deterministic Caddy site files for managed apps.

Rules:

```txt
Only update a Caddy site if the effective generated site content changed.
If no site files changed, do not reload Caddy.
Validate Caddy configuration before reload.
Do not reload Caddy after failed validation.
If a managed container was removed, remove the corresponding managed Caddy site.
If a managed container was stopped but still exists, do not remove the Caddy site solely because it is stopped.
If a container restarts with unchanged routing metadata, do not update Caddy.
If ports, labels, domain, upstream, TLS, or generated site content changes, update the site and reload after validation.
```

Caddy API policy:

```txt
Caddy API is read-only for now, except validate/reload workflows.
Do not use the Caddy API as the primary write path for route mutation.
Generated site files are the primary write path.
```

---

# Podman Rules

Podman is the only v1 container runtime.

Cadman may read Podman state through:

```txt
Podman CLI
Podman socket
```

Native Linux Cadman may use CLI or socket.

Containerized Cadman must use the Podman socket only.

Containerized Cadman must not:

```txt
shell out to host podman
assume host binaries exist
depend on host filesystem paths unless explicitly mounted/configured
```

---

# Project Config

`cadman init` creates a project config.

Default:

```sh
cadman init
```

creates:

```txt
cadman.toml
```

YAML mode:

```sh
cadman init --yaml
```

creates:

```txt
cadman.yaml
```

Rules:

```txt
TOML is default.
YAML is opt-in.
Do not overwrite existing cadman.toml or cadman.yaml unless an explicit force flag is provided.
```

---

# v0.1.0 Scope

Goal: establish the Rust foundation.

In scope:

```txt
CLI-only runtime
architecture boundaries
cadman caddy
cadman podman
cadman compose
cadman containers
RunMode support: current, cadman, root
inspect running Podman containers
collect ports, health status, labels
return structured JSON payloads
use veltrix and scriba from crates.io
```

Out of scope:

```txt
API as public surface
Web UI as public surface
daemon mode as required runtime
Docker support
automatic Caddy site generation
project registry
long-running reconciler
```

---

# v1 Command Direction

Target v0.1.0 commands:

```txt
cadman init
cadman scan
cadman containers
cadman config
cadman registry
cadman add
cadman remove
cadman status
cadman doctor
cadman reconcile
cadman self
cadman caddy
cadman podman
cadman compose
```

Command meanings:

```txt
cadman init        create cadman.toml or cadman.yaml with --yaml
cadman scan        discover project files and optionally register them
cadman containers  list visible Podman containers, state, ports, health, labels
cadman config      inspect and initialize config.toml
cadman registry    inspect and mutate registry.toml
cadman add         register the current project in registry.toml
cadman remove      remove a project/app from registry.toml
cadman status      show runtime, app, container, route, site, and state status
cadman doctor      diagnose environment, permissions, Podman, Caddy, config/state
cadman reconcile   collect desired routes, apply Caddy sites, reload, update state
cadman self        install, update, uninstall, and healthcheck Cadman
cadman caddy       controlled Caddy wrapper
cadman podman      controlled Podman wrapper
cadman compose     controlled Podman Compose workflow wrapper
```

Deferred v1+ command ideas:

```txt
cadman up
cadman down
cadman inspect
cadman logs
cadman serve
```

The detailed v0.1.0 CLI contract lives in:

```txt
docs/CLI_CONTRACT.md
```

---

# `cadman containers` Rules

`cadman containers` lists Podman containers visible to the selected Cadman RunMode.

It should show:

```txt
requested RunMode
effective RunMode
effective user
Podman discovery source
container name
container ID
image
state
health status
ports
labels when requested
```

Suggested flags:

```txt
--all
--running
--stopped
--labels
```

Default behavior:

```txt
list running containers
```

Text output should include runtime context before the table.

JSON output should include:

```txt
runtime
containers
```

JSON output should always include labels.

Text output may omit labels unless `--labels` is passed.

---

# Required Command Flow

Every command should follow this shape:

```rust
pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    // CLI-only parsing/intent conversion
    core::usecases::some_feature::run(ctx, request).await
}
```

The CLI creates intent and calls core. Core performs the workflow and renders output.

Do not use this older pattern:

```rust
pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    let report = core::usecases::some_feature::run(ctx, request).await?;
    render_report(ctx, report)
}
```

Core owns rendering.

---

# Where Logic Belongs

| Concern                           | Location                               |
| --------------------------------- | -------------------------------------- |
| Argument parsing                  | `cli/`                                 |
| CLI flag conflict checks          | `cli/`                                 |
| Command dispatch                  | `cli/`                                 |
| User-facing rendering             | `core/` using Scriba                   |
| Workflow orchestration            | `core/usecases/`                       |
| Authorization / effective RunMode | `core` calling `engine` runtime policy |
| Podman discovery                  | `engine/capabilities/podman/`          |
| Caddy integration                 | `engine/capabilities/caddy/`           |
| Registry handling                 | `engine/registry/`                     |
| State handling                    | `engine/state/`                        |
| Config handling                   | `engine/config/`                       |
| Reconciliation                    | `engine/capabilities/reconcile/`       |
| Daemon/API/Web UI capabilities    | `engine/capabilities/*`                |
| Data models                       | `engine/models/`                       |
| Errors                            | `engine/errors/`                       |

---

# Anti-Patterns to Reject

Reject this in `cli/`:

```rust
PodmanCliClient::new(...)
podman.containers()
serde_json::to_value(container)
std::fs::read_to_string(...)
toml::from_str(...)
std::process::Command::new("podman")
scriba::Table::new(...)
Ui::new().print(...)
```

These belong in `engine` or `core`, depending on concern.

Reject business-rule code in `cli/`:

```rust
if container.status == "running" { ... }
if site_hash_changed { ... }
if effective_mode == Root { ... }
if registry.contains(app) { ... }
```

CLI may validate CLI syntax only, for example:

```rust
if args.all && args.running {
    return Err(ErrorCode::UserInvalidArgument.error()
        .with_context("conflict", "--all cannot be used with --running"));
}
```

---

# Error Rules

Use structured engine errors.

Errors belong in:

```txt
engine/errors/
  kind.rs
  code.rs
  mod.rs
```

Use:

```rust
crate::engine::errors::{Error, ErrorCode, ErrorKind, Result}
```

Do not use:

```rust
Box<dyn Error>
anyhow
eyre
string-only errors
```

Unless explicitly approved for a narrow boundary.

Errors should include useful context:

```rust
ErrorCode::PodmanCommandFailed
    .error()
    .with_context("command", "podman ps --all --format json")
    .with_context("error", err.to_string())
```

---

# Security Rules

Security is a hard requirement.

Do not:

```txt
silently escalate privileges
turn Cadman into a root shell wrapper
run arbitrary Podman commands as root by default
write non-Cadman Caddy files
reload Caddy after failed validation
log secrets, tokens, passwords, or credential-bearing socket paths
use Docker APIs or Docker socket in v1
bind managed container ports publicly by default
```

Prefer:

```txt
least privilege
rootless operation
explicit authority
scoped RunMode behavior
deterministic config generation
validation before mutation
structured errors
auditable state transitions
```

---

# Build Principles

When implementing Cadman:

```txt
Keep v1 narrow and maintainable.
Preserve dependency flow: cli -> core -> engine.
Keep CLI thin.
Put user-facing workflow and rendering in core.
Put capabilities and business logic in engine.
Use Veltrix for system integration where possible.
Use Scriba for core-rendered UI where possible.
Avoid unnecessary Caddy reloads.
Treat registry/state/config as authoritative.
Treat Caddy site files as generated output.
Do not add Docker behavior before v1.
Use structured errors.
Avoid silent privilege escalation.
```

---

# Final Rule

If a function needs to know about Podman, Caddy, Cadman state, registry contents, project files, RunMode authorization, or reconciliation, it does **not** belong in `cli`.

```txt
CLI captures intent.
Core renders experience.
Engine owns Cadman behavior.
```
