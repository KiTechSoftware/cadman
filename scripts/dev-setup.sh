#!/usr/bin/env bash
set -euo pipefail

echo ">> ensuring rustfmt, clippy, llvm-tools-preview"
rustup component add rustfmt clippy llvm-tools-preview

if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
  echo ">> installing cargo-llvm-cov"
  cargo install cargo-llvm-cov --locked
else
  echo ">> cargo-llvm-cov already installed"
fi

echo "✅ dev setup complete"
