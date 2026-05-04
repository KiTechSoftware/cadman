#!/usr/bin/env sh
set -e

# Start caddy in background
caddy run --config /etc/cadman/caddy/Caddyfile --resume &
CADDY_PID=$!

# Start cadman in foreground (so container exits if cadman dies)
exec cadman --config /etc/cadman/cadman.toml
