#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/libs/common.sh
source "${SCRIPT_DIR}/libs/common.sh"

os_distro="$(detect_os)"
os="${os_distro%%|*}"
pkg_mgr="$(detect_pkg_mgr "$os")"

ensure_dev_tools "$pkg_mgr"

install_container_engine "$os" "$pkg_mgr"
install_optional_tools

