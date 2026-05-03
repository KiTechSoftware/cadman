# Cadman v0.1.0 Release Notes

Cadman v0.1.0 is the first MVP release of the rebuilt Cadman workflow.

Cadman is Linux-only, Podman-first, and Caddy-first. Docker is not supported in v0.1.0. API, Web UI, and daemon/live watcher modes are not public or required runtime surfaces yet.

## Highlights

- Initialize Cadman project configs with `cadman init`.
- Discover project configs with `cadman scan`.
- Register managed apps with `cadman add` and `cadman scan --add`.
- Discover explicit labeled Podman containers during `cadman reconcile`.
- Generate deterministic Cadman-managed Caddy site files.
- Validate Caddy before reload.
- Reload only when generated files changed and validation passed.
- Record app, route, container, and site state in `state.json`.
- Inspect environment and state with `cadman doctor` and `cadman status`.
- Plan lifecycle operations with `cadman --dry-run self install`.

## Command Surface

The frozen v0.1.0 command contract is documented in `docs/CLI_CONTRACT.md`.

Supported command groups:

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

## Validation

Release-polish checks run locally:

```txt
cargo fmt -- --check
cargo clippy --all-targets --all-features
cargo test -- --test-threads=1
cargo doc --no-deps
scripts/smoke-v0.1.0.sh
```

`cargo clippy --all-targets --all-features` exits successfully but currently reports warnings for mechanical cleanup opportunities, including derivable `Default` impls, collapsible `if` blocks, needless borrows in Scriba JSON calls, and test mutex guards held across awaits.

## Known Host Requirements

Non-dry reconcile requires a prepared host:

```txt
Podman installed and working
Caddy installed and configured to include Cadman generated sites
cadman user/group available for system installs
rootless Podman subuid/subgid ranges configured where needed
working directory accessible to the selected run mode
```

## Out Of Scope

```txt
Docker runtime support
Docker socket support
Traefik integration
Kubernetes
Windows/macOS support
public API surface
public Web UI surface
required daemon/live watcher runtime
implicit sudo escalation
Caddy API route mutation as primary write path
```
