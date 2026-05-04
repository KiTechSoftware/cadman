#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKSPACE="$ROOT/workspace"
TMP="$(mktemp -d)"
STATE_HOME="$TMP/state"
CONFIG_HOME="$TMP/config"
CADMAN_BIN="${CADMAN_BIN:-}"

cleanup() {
  rm -rf "$TMP"
}
trap cleanup EXIT

run_cadman() {
  if [[ -n "$CADMAN_BIN" ]]; then
    "$CADMAN_BIN" "$@"
  else
    cargo run --quiet --manifest-path "$WORKSPACE/Cargo.toml" -- "$@"
  fi
}

mkdir -p "$STATE_HOME" "$CONFIG_HOME" "$TMP/project"

export XDG_STATE_HOME="$STATE_HOME"
export XDG_CONFIG_HOME="$CONFIG_HOME"

echo "== build =="
cargo build --manifest-path "$WORKSPACE/Cargo.toml"

echo "== version =="
run_cadman --version

echo "== config =="
run_cadman --cwd "$TMP/project" config path
run_cadman --cwd "$TMP/project" registry list

if [[ "${CADMAN_SMOKE_MUTATE:-0}" == "1" ]]; then
  echo "== project =="
  run_cadman --cwd "$TMP/project" config init
  run_cadman --cwd "$TMP/project" init
  run_cadman --cwd "$TMP/project" add
  run_cadman --cwd "$TMP/project" registry list

  echo "== discovery =="
  run_cadman --cwd "$TMP/project" scan "$TMP"
else
  echo "== project =="
  echo "Skipping mutating config/init/add smoke. Set CADMAN_SMOKE_MUTATE=1 on a prepared user-scope environment."
fi

echo "== reconcile/status =="
run_cadman --cwd "$TMP/project" --dry-run reconcile
run_cadman --cwd "$TMP/project" doctor
run_cadman --cwd "$TMP/project" status

echo "smoke v0.1.0 passed"
