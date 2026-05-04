# Getting Started

Cadman is a Linux-only, Podman-first, Caddy-first workflow tool.

Build the development binary:

```sh
cd workspace
cargo build
```

Check the environment:

```sh
cargo run -- --dry-run self install
cargo run -- self healthcheck
cargo run -- doctor
```

Create and register a project:

```sh
cargo run -- init
cargo run -- add
cargo run -- registry list
```

Reconcile routes:

```sh
cargo run -- --dry-run reconcile
cargo run -- reconcile
cargo run -- status
```

For the full v0.1.0 command contract, see `docs/CLI_CONTRACT.md`.
