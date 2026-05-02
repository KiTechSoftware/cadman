#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
PORT="${PORT:-8000}"
TAG="mkdocs-custom"

"${REPO_ROOT}/scripts/libs/ctr.sh" run --rm -it -p "${PORT}:8000" -v "${REPO_ROOT}/docs:/docs:Z" "$TAG" serve -a 0.0.0.0:8000