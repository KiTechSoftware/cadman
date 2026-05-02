#!/usr/bin/env bash
set -euo pipefail

pick_engine() {
  local want="${1:-auto}"

  case "$want" in
    auto|"")
      if command -v podman >/dev/null 2>&1; then
        echo "podman"
        return 0
      fi
      if command -v docker >/dev/null 2>&1; then
        echo "docker"
        return 0
      fi
      ;;
    podman)
      if command -v podman >/dev/null 2>&1; then
        echo "podman"
        return 0
      fi
      echo "CONTAINER_ENGINE=podman was requested, but podman is not installed or not in PATH" >&2
      exit 1
      ;;
    docker)
      if command -v docker >/dev/null 2>&1; then
        echo "docker"
        return 0
      fi
      echo "CONTAINER_ENGINE=docker was requested, but docker is not installed or not in PATH" >&2
      exit 1
      ;;
    *)
      echo "Invalid CONTAINER_ENGINE='$want'. Valid values: auto, podman, docker" >&2
      exit 1
      ;;
  esac

  echo "No container engine found (looked for podman, docker)" >&2
  exit 1
}

if [[ -z "${CONTAINER_ENGINE:-}" ]]; then
  echo "Hint: set CONTAINER_ENGINE=auto|podman|docker to skip auto-detection messaging." >&2
fi

ENGINE="$(pick_engine "${CONTAINER_ENGINE:-auto}")"
exec "$ENGINE" "$@"