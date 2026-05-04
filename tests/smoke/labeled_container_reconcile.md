# Labeled Container Reconcile Smoke

## Goal

Verify the explicit labeled container workflow.

## Setup

```sh
podman rm -f cadman-demo 2>/dev/null || true
podman run -d \
  --name cadman-demo \
  -p 127.0.0.1:8080:80 \
  --label cadman.enable=true \
  --label cadman.host=demo.local,www.demo.local \
  --label cadman.port=80 \
  docker.io/library/nginx:alpine
```

## Steps

```sh
cadman containers --all --labels
cadman --dry-run reconcile
cadman reconcile
cadman registry show cadman-demo
cadman status cadman-demo
curl -H 'Host: demo.local' http://127.0.0.1
```

## Expected Result

```txt
containers shows cadman-demo
dry-run reconcile plans a podman-label route
reconcile registers cadman-demo in registry.toml
reconcile writes a generated Caddy site file
Caddy validates before reload
status shows route/site/container state
curl reaches the container through Caddy when host Caddy is configured to include Cadman sites
```

## Cleanup

```sh
podman rm -f cadman-demo
cadman registry remove cadman-demo
cadman reconcile
```
