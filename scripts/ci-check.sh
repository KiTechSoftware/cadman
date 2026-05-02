#!/usr/bin/env bash
set -euo pipefail

THRESHOLD_LINES="${THRESHOLD_LINES:-70}"

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

echo ">> fmt check"
cargo fmt --manifest-path "${REPO_ROOT}/workspace/Cargo.toml" --all -- --check

echo ">> clippy (deny warnings)"
cargo clippy --manifest-path "${REPO_ROOT}/workspace/Cargo.toml" --workspace --all-features --all-targets -- -D warnings

echo ">> tests"
cargo test --manifest-path "${REPO_ROOT}/workspace/Cargo.toml" --workspace --all-features

echo ">> coverage (fail-under-lines=${THRESHOLD_LINES})"
cargo llvm-cov \
  --manifest-path "${REPO_ROOT}/workspace/Cargo.toml" \
  --workspace \
  --all-features \
  --fail-under-lines "${THRESHOLD_LINES}" \
  --no-report

echo ">> license check"
cargo deny check --manifest-path "${REPO_ROOT}/workspace/Cargo.toml" --config workspace/deny.toml

echo "✅ all local checks passed"