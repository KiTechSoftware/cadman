# Getting Started

Cadman is a Linux-only, Podman-first, Caddy-first workflow tool.

## Prerequisites

- A Linux host with Podman installed and usable by the Cadman user.
- Caddy installed for site validation and reloads.

## Install

Install a packaged Cadman binary for your distribution, or download the prebuilt release from the project's release page. If you are a developer, see the developer docs for build instructions.

## Common user workflows

- Initialize a project in a repository (creates cadman.toml):

```sh
cadman init
```

- Register the current project with the Cadman registry:

```sh
cadman add
cadman registry list
```

- Run a dry-run reconcile to see planned changes without mutating anything:

```sh
cadman reconcile --dry-run
```

- Apply reconcile to generate site files and (if valid) request a Caddy reload:

```sh
cadman reconcile
```

- Inspect runtime status:

```sh
cadman status
```
