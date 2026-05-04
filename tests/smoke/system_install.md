# System Install Smoke

## Goal

Verify system install planning and host prerequisites.

## Steps

```sh
cadman --mode root --dry-run self install
cadman self healthcheck
cadman doctor
```

If the dry-run plan is acceptable and the host is prepared:

```sh
sudo cadman --mode root self install
cadman self healthcheck
cadman doctor
cadman containers --all
cadman --dry-run reconcile
```

## Expected Result

```txt
dry-run self install shows system target steps
self healthcheck reports cadman user/group state
doctor reports Podman, Caddy, systemd, config, registry, state, and Caddy sites diagnostics
containers lists only authorized container scopes
dry-run reconcile completes
```

## Host Requirements

```txt
Linux host
Podman installed
Caddy installed
cadman user/group available or install plan can create them
current user in cadman or admin group for system-scope visibility
rootless Podman subuid/subgid ranges configured for cadman user where needed
Caddy configured to include Cadman generated site files
```
