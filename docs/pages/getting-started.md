# Getting Started

This guide follows the current **SRS**. If something differs in your build, trust the CLI `--help` first.

## Prerequisites

- OS: macOS, Debian/Ubuntu, or Fedora (WSL partial planned).
- Tools: Podman, Podman Compose, Caddy.
- Permissions: ability to run Podman and manage files in your project directories.

## Install

> Until a formula/package exists, build from source.

```bash
# Clone and build (example)
git clone https://github.com/kitechsoftware/cadman
cd cadman
cargo build --release
# Add target/release/cadman to your PATH
```

When packages are available:
- macOS (Homebrew): `brew install cadman` (planned)
- Linux (apt/dnf): coming later

## Quick Start

1) **Create a project folder**

```bash
mkdir -p ~/apps/blog && cd ~/apps/blog
```

2) **Create a minimal `cadman.yaml`**

```yaml
name: blog
containers:
  default:
    name: blog-container
    image: ghcr.io/example/blog:latest
    detached: true
    restart: always
    ports:
      - "127.0.0.1:3000:80"
```

3) **Initialize and start**

```bash
cadman init
cadman up
```

4) **Expose via Caddy (optional)**

Define `caddy.routes` in `cadman.yaml` and run:
```bash
cadman daemon reload   # validates and applies Caddy config without downtime
```

## Next steps

- Explore the [Commands](commands/index.md)
- Read the [Architecture](architecture/index.md)
