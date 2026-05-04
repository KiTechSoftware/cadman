# Cadman

Cadman is a Linux-only, Podman-first, Caddy-first workflow tool for discovering workloads, registering apps, generating Caddy site files, and reconciling route state.

Cadman v0.1.0 does not support Docker, Traefik, Kubernetes, macOS, Windows, a public API, or a public Web UI.

## MVP Quickstart

Build the development binary:

```sh
cd workspace
cargo build
```

Check the install environment:

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

Discover project configs from configured or explicit roots:

```sh
cargo run -- config init
cargo run -- scan
cargo run -- scan --add /path/to/apps
```

Run the reconcile loop:

```sh
cargo run -- --dry-run reconcile
cargo run -- reconcile
cargo run -- status
```

Labeled container workflow:

```sh
podman run -d \
  --name cadman-demo \
  -p 127.0.0.1:8080:80 \
  --label cadman.enable=true \
  --label cadman.host=demo.local,www.demo.local \
  --label cadman.port=80 \
  docker.io/library/nginx:alpine

cargo run -- reconcile
cargo run -- registry show cadman-demo
cargo run -- status cadman-demo
```

Reconcile discovers labeled running containers, registers them as label-sourced apps, writes deterministic Cadman-managed Caddy site files, validates Caddy before reload, reloads only when files changed, and records observed state.

See [docs/CLI_CONTRACT.md](docs/CLI_CONTRACT.md) for the frozen v0.1.0 command contract.
