## Architecture Boundary

Cadman uses a sovereign architecture with one primary dependency direction:

```txt
cli -> core -> engine
```

### Source layout

```txt
workspace/
  src/
    main.rs
    lib.rs

    cli/        # User-facing CLI surface
    core/       # Application orchestration and workflows
    engine/     # Sovereign Cadman capabilities, models, errors, config, state
```

### Dependency rules

```txt
engine imports nothing from cli or core
core imports engine
cli imports core
cli may import engine error types for error mapping and presentation
```

No top-level modules should be created for:

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

## Layer Responsibilities

### `cli/`

The CLI is the user-facing control surface.

It owns:

```txt
argument parsing
command dispatch
terminal rendering
output formatting
user-facing error display
```

It must not own:

```txt
business logic
Podman discovery logic
Caddy integration logic
registry/state mutation logic
reconciliation logic
daemon loops
API runtime behavior
Web UI runtime behavior
```

CLI may call `core` workflows and may use `engine::errors` for error mapping.

### `core/`

Core owns orchestration.

It owns:

```txt
use-cases
workflow coordination
ordering of engine operations
command-level application behavior
authorization-aware operation routing
```

It must not own:

```txt
terminal rendering
HTTP request/response mapping
Web UI rendering
raw Podman/Caddy implementation details
state file serialization details
```

Core calls engine capabilities and returns structured results.

### `engine/`

Engine is Cadman’s sovereign layer.

It owns:

```txt
domain models
typed errors
constants
config handling
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
```

Engine must remain independent of CLI/core presentation concerns.

## Engine Capabilities Layout

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
  config/          # config.toml
  registry/        # registry.toml
  state/           # state.json
  constants/       # labels, paths, env vars, defaults
  system/          # filesystem/process primitives only
```

## Scriba Boundary

Use `scriba` wherever Cadman needs user-facing output, structured rendering, tables, prompts, logging, envelopes, or output formats.

Preferred uses:

```txt
CLI table rendering
JSON/text/markdown output formatting
terminal logging
interactive prompts
output envelopes
color policy
verbosity/quiet handling
```

Rules:

```txt
Use scriba instead of hand-rolled terminal rendering.
Use scriba instead of ad hoc table formatting.
Use scriba instead of custom prompt helpers.
Use scriba output/envelope types for command output where practical.
Keep scriba-facing rendering mostly in cli.
Engine may define serializable data models, but should not render terminal output.
Core should return structured data, not formatted strings.
```

Boundary:

```txt
cli -> scriba      allowed
core -> scriba     avoid unless carrying runtime/output config only
engine -> scriba   avoid for rendering; acceptable only for shared runtime output config types if already chosen
```

Preferred pattern:

```txt
engine returns model
core returns model/result
cli renders model with scriba
```

Example:

```txt
engine::capabilities::podman::list_containers()
  -> ContainerListReport

core::usecases::containers::list()
  -> ContainerListReport

cli::cmd::containers::run()
  -> render ContainerListReport using scriba
```

## Veltrix Boundary

Use `veltrix` wherever Cadman interacts with system, process, Podman, Caddy, users, groups, paths, or Linux platform primitives.

Preferred uses:

```txt
Podman CLI access
Podman socket access
Caddy CLI/API access where available
process execution
UID/GID/user/group lookup
system/user path resolution
Linux runtime helpers
Unicode/constants already provided by veltrix
```

Rules:

```txt
Use veltrix instead of raw std::process where practical.
Use veltrix instead of manually shelling out to podman/caddy when typed clients exist.
Use veltrix instead of custom UID/GID lookup logic where possible.
Use veltrix path helpers instead of hard-coded Linux paths where possible.
Do not duplicate functionality already provided by veltrix.
```

Boundary:

```txt
engine -> veltrix  allowed and preferred
core -> veltrix    avoid
cli -> veltrix     avoid, except simple CLI-only environment checks if unavoidable
```

Preferred pattern:

```txt
cli parses command
core chooses workflow
engine capability uses veltrix
engine returns typed Cadman model/result
cli renders using scriba
```

## System Primitive Rule

Low-level system wrappers may exist under:

```txt
engine/system/
```

But only for reusable primitives such as:

```txt
filesystem helpers
path helpers
process wrapper abstraction
atomic file writes
locks
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

## Hard Rules

```txt
Do not put business logic in cli.
Do not render terminal output in engine.
Do not call Podman/Caddy directly from cli.
Do not make core depend on cli.
Do not make engine depend on core.
Do not create top-level daemon/api/webui layers.
Do not bypass veltrix when a veltrix client/helper exists.
Do not bypass scriba for CLI output formatting unless the output is raw passthrough from a wrapped command.
Do not silently escalate privileges.
Do not use Docker APIs or Docker socket in v1.
```

## One-Line Principle

```txt
CLI renders with scriba, core orchestrates, engine owns Cadman behavior and uses veltrix for system integration.
```

Use this stronger instruction block:

## Strict Architecture Rule: No Logic in CLI

Cadman uses this dependency and responsibility flow:

```txt
user -> cli -> core -> engine
```

The CLI is a **thin adapter only**.

### CLI may do only these things

```txt
parse command-line arguments
validate CLI-only flag conflicts
convert CLI args into core request structs
call core workflows
render returned data using scriba
map errors to user-facing output
return exit codes
```

### CLI must not do these things

```txt
call Podman directly
call Caddy directly
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
```

## Required Flow

Every command must follow this shape:

```rust
pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    let request = SomeRequest::from(args);
    let report = core::usecases::some_feature::run(ctx, request).await?;
    render_report(ctx, report)
}
```

The CLI creates a request and renders a report. Nothing else.

## Where Logic Belongs

| Concern                           | Location                               |
| --------------------------------- | -------------------------------------- |
| Argument parsing                  | `cli/`                                 |
| CLI flag conflict checks          | `cli/`                                 |
| Terminal rendering                | `cli/` using `scriba`                  |
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

## Required Pattern

Use request/report models.

```txt
cli args -> core request -> engine capability -> engine model/report -> core returns report -> cli renders
```

Example for `cadman containers`:

```txt
cli/cmd/containers.rs
  parses: --all, --running, --stopped, --labels
  validates: mutually exclusive flags only
  creates: ContainerListRequest
  calls: core::usecases::containers::list(ctx, request)
  renders: ContainerListReport with scriba

core/usecases/containers.rs
  receives ContainerListRequest
  determines workflow ordering
  calls engine::capabilities::podman::list_containers(...)
  returns ContainerListReport

engine/capabilities/podman/
  talks to Podman using veltrix
  extracts container metadata
  applies runtime/podman discovery rules
  returns typed ContainerListReport
```

## Anti-Patterns to Reject

Reject code like this in `cli/`:

```rust
PodmanCliClient::new(...)
podman.containers()
serde_json::to_value(container)
std::fs::read_to_string(...)
toml::from_str(...)
std::process::Command::new("podman")
```

These belong in `engine`, not CLI.

Reject business-rule code in CLI, such as:

```rust
if container.status == "running" { ... }
if site_hash_changed { ... }
if effective_mode == Root { ... }
if registry.contains(app) { ... }
```

The CLI may validate only CLI syntax/flag conflicts, for example:

```rust
if args.all && args.running {
    return Err(ErrorCode::UserInvalidArgument.error()
        .with_context("conflict", "--all cannot be used with --running"));
}
```

## Scriba and Veltrix Placement

Use `scriba` primarily in `cli` for rendering:

```txt
cli -> scriba
```

Use `veltrix` primarily in `engine` for system integration:

```txt
engine -> veltrix
```

Do not use `veltrix::services::podman::*` in `cli`.

Do not use `std::process::Command` in `cli`.

## Final Rule

If a function needs to know anything about Podman, Caddy, Cadman state, registry contents, project files, RunMode authorization, or reconciliation, it does **not** belong in `cli`.

CLI is the shell. Core is the workflow. Engine is the product.
