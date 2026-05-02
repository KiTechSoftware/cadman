#!/usr/bin/env bash
set -euo pipefail

THRESHOLD_LINES="${THRESHOLD_LINES:-70}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

echo ">> coverage (fail-under-lines=${THRESHOLD_LINES})"
cargo llvm-cov \
  --manifest-path "${REPO_ROOT}/workspace/Cargo.toml" \
  --workspace \
  --all-features \
  --fail-under-lines "${THRESHOLD_LINES}" \
  --no-report