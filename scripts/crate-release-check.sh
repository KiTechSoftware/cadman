#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
# shellcheck source=./libs/common.sh
source "${SCRIPT_DIR}/libs/common.sh"

CRATE_PATH="${REPO_ROOT}/workspace/crates/commit-wizard"

bold "🧙‍♂️ Commit Wizard Crate Release Checks"
echo

# 1. Ensure all tests pass
info "Running tests..."
cargo test \
  --manifest-path "${CRATE_PATH}/Cargo.toml" \
  --lib \
  --all-features
ok "Tests passed"
echo

# 2. Check for warnings
info "Running clippy (deny warnings)..."
cargo clippy \
  --manifest-path "${CRATE_PATH}/Cargo.toml" \
  --all-targets \
  --all-features \
  -- -D warnings
ok "Clippy checks passed"
echo

# 3. Check documentation builds
info "Building documentation..."
cargo doc \
  --manifest-path "${CRATE_PATH}/Cargo.toml" \
  --no-deps \
  --all-features
ok "Documentation built successfully"
echo

# 4. Verify dry-run publish
info "Verifying crate with dry-run publish..."
cd "${CRATE_PATH}"
cargo publish --dry-run
ok "Dry-run publish successful"
cd - > /dev/null
echo

# 5. Final verification - check dependencies
info "Verifying crate tree..."
cargo tree \
  --manifest-path "${CRATE_PATH}/Cargo.toml" \
  --depth 2 \
  --duplicates
ok "Dependency tree verified"
echo

bold "✅ All crate release checks passed!"