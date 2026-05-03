#!/usr/bin/env bash
set -euo pipefail

TAG="commit-wizard"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

"${REPO_ROOT}/scripts/libs/ctr.sh" build -t "$TAG" -f "${REPO_ROOT}/docker/base/Dockerfile" "${REPO_ROOT}/workspace"
"${REPO_ROOT}/scripts/libs/ctr.sh" run --rm -it --user "$(id -u)":"$(id -g)" -v "$(pwd)":/app:Z -w /app "$TAG" "$@"
