# User Install Smoke

## Goal

Verify the user-facing install and project registration loop.

## Steps

```sh
cd workspace
cargo build
target/debug/cadman --version
target/debug/cadman --dry-run self install
target/debug/cadman self healthcheck
```

Create a temporary project:

```sh
tmp="$(mktemp -d)"
cd "$tmp"
/path/to/cadman config path
/path/to/cadman config init
/path/to/cadman init
/path/to/cadman add
/path/to/cadman registry list
/path/to/cadman --dry-run reconcile
/path/to/cadman doctor
/path/to/cadman status
```

## Expected Result

```txt
cadman --version prints 0.1.0
self install dry-run renders an install plan
self healthcheck reports install facts
init creates cadman.toml
add registers the project
registry list shows the app
dry-run reconcile completes without mutation
doctor and status render clean reports
```
