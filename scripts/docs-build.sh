#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
TAG="mkdocs-custom"

"${REPO_ROOT}/scripts/libs/ctr.sh" build -t "$TAG" -f "${REPO_ROOT}/docker/docs/Dockerfile" "${REPO_ROOT}/docs"