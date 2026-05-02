#!/usr/bin/env bash
set -euo pipefail

bold()  { printf "\033[1m%s\033[0m\n" "$*"; }
info()  { printf "➜ %s\n" "$*"; }
ok()    { printf "✔ %s\n" "$*"; }
warn()  { printf "⚠ %s\n" "$*"; }
err()   { printf "✖ %s\n" "$*" >&2; }

has_cmd() {
  command -v "$1" >/dev/null 2>&1
}

need_cmd() {
  local cmd="$1"
  local hint="${2:-}"
  if ! has_cmd "$cmd"; then
    err "Required command not found: $cmd"
    if [[ -n "$hint" ]]; then
      printf "%s\n" "$hint" >&2
    fi
    exit 1
  fi
}

need_sudo() {
  if [ "$(id -u)" -ne 0 ]; then
    if ! has_cmd sudo; then
      err "This script needs administrative privileges but 'sudo' is not available."
      exit 1
    fi
    echo sudo
  else
    echo ""
  fi
}

detect_os() {
  local os="unknown" distro="unknown"
  case "$(uname -s)" in
    Darwin) os="macos" ;;
    Linux)  os="linux" ;;
  esac

  if [ "$os" = "linux" ] && [ -r /etc/os-release ]; then
    # shellcheck disable=SC1091
    . /etc/os-release
    distro="${ID:-unknown}"
  fi

  printf "%s|%s" "$os" "$distro"
}

detect_pkg_mgr() {
  local os="$1"

  if [ "$os" = "macos" ]; then
    if has_cmd brew; then echo "brew"; return; fi
    if has_cmd port; then echo "port"; return; fi
    echo "none"
    return
  fi

  for pm in apt dnf yum zypper pacman; do
    if has_cmd "$pm"; then
      echo "$pm"
      return
    fi
  done

  echo "none"
}

ensure_curl() {
  if has_cmd curl; then
    return
  fi

  local pm="$1"
  local s
  s="$(need_sudo)"

  info "Installing curl with $pm ..."
  case "$pm" in
    apt)    $s apt-get update -y && $s apt-get install -y curl ;;
    dnf)    $s dnf install -y curl ;;
    yum)    $s yum install -y curl ;;
    zypper) $s zypper --non-interactive install curl ;;
    pacman) $s pacman -Sy --noconfirm curl ;;
    brew)   brew install curl ;;
    *)
      err "Please install curl first, then re-run."
      exit 1
      ;;
  esac

  ok "curl installed."
}

ensure_path_snippet() {
  local rc
  for rc in "$HOME/.bashrc" "$HOME/.zshrc"; do
    [ -f "$rc" ] || continue
    if ! grep -q 'export PATH="$HOME/.cargo/bin:$PATH"' "$rc" 2>/dev/null; then
      echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> "$rc"
    fi
  done
}

install_rustup() {
  if has_cmd rustup && has_cmd cargo; then
    ok "rustup/cargo already installed."
    return
  fi

  local pm="$1"
  ensure_curl "$pm"

  info "Installing rustup ..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

  # shellcheck disable=SC1091
  . "$HOME/.cargo/env"
  ensure_path_snippet
  ok "rustup/cargo installed."
}

ensure_rust_component() {
  local component="$1"
  if rustup component list --installed | grep -qx "$component"; then
    ok "rust component already installed: $component"
    return
  fi

  info "Installing rust component: $component"
  rustup component add "$component"
  ok "installed rust component: $component"
}

install_cargo_binary() {
  local crate="$1"
  local bin="${2:-$1}"

  if has_cmd "$bin"; then
    ok "$bin already installed."
    return
  fi

  info "Installing $crate ..."
  cargo install "$crate" --locked
  ok "$bin installed."
}

install_mise() {
  if has_cmd mise; then
    ok "mise already installed."
    return 0
  fi

  local os_distro os pm
  os_distro="$(detect_os)"
  os="${os_distro%%|*}"
  pm="$(detect_pkg_mgr "$os")"

  info "Installing mise ..."

  case "$os:$pm" in
    macos:brew)
      brew install mise
      ;;
    linux:apt)
      local s
      s="$(need_sudo)"
      $s apt-get update -y
      $s apt-get install -y gpg curl
      curl https://mise.run | sh
      ;;
    linux:dnf|linux:yum|linux:zypper|linux:pacman|macos:port|linux:none|macos:none)
      curl https://mise.run | sh
      ;;
    *)
      curl https://mise.run | sh
      ;;
  esac

  if has_cmd mise || [[ -x "$HOME/.local/bin/mise" ]]; then
    ok "mise installed."
    return 0
  fi

  warn "mise installation finished, but 'mise' is not yet on PATH in this shell."
  warn "Restart your shell or add the mise bin directory to PATH."
}

install_lefthook() {
  if has_cmd lefthook; then
    ok "lefthook already installed."
    return 0
  fi

  local os_distro os pm
  os_distro="$(detect_os)"
  os="${os_distro%%|*}"
  pm="$(detect_pkg_mgr "$os")"

  info "Installing lefthook ..."

  if has_cmd mise; then
    mise use -g lefthook@latest
  elif has_cmd go; then
    GO111MODULE=on go install github.com/evilmartians/lefthook/v2@latest
  else
    case "$os:$pm" in
      macos:brew)
        brew install lefthook
        ;;
      linux:apt)
        local s
        s="$(need_sudo)"
        $s apt-get update -y
        $s apt-get install -y lefthook
        ;;
      linux:yum)
        local s
        s="$(need_sudo)"
        $s yum install -y lefthook
        ;;
      linux:dnf)
        local s
        s="$(need_sudo)"
        $s dnf install -y lefthook
        ;;
      linux:zypper)
        local s
        s="$(need_sudo)"
        $s zypper --non-interactive install lefthook
        ;;
      linux:pacman)
        local s
        s="$(need_sudo)"
        $s pacman -Sy --noconfirm lefthook
        ;;
      *)
        err "No supported automatic Lefthook installer configured for this system."
        err "Use mise ('mise use lefthook@latest'), Go install, or install Lefthook manually."
        exit 1
        ;;
    esac
  fi

  if has_cmd lefthook || [[ -x "$HOME/.local/share/mise/shims/lefthook" ]] || [[ -x "$HOME/go/bin/lefthook" ]]; then
    ok "lefthook installed."
    return 0
  fi

  err "lefthook installation failed."
  exit 1
}

install_optional_just() {
  if has_cmd just; then
    ok "just already installed."
    return 0
  fi

  local os_distro os pm
  os_distro="$(detect_os)"
  os="${os_distro%%|*}"
  pm="$(detect_pkg_mgr "$os")"

  info "Installing optional tool: just ..."

  case "$os:$pm" in
    macos:brew)
      brew install just
      ;;
    linux:apt|linux:dnf|linux:yum|linux:zypper|linux:pacman)
      if has_cmd cargo-binstall; then
        cargo binstall just --no-confirm
      else
        cargo install --locked just
      fi
      ;;
    *)
      cargo install --locked just
      ;;
  esac

  if has_cmd just; then
    ok "just installed."
  else
    warn "just installation failed or is not on PATH yet."
  fi
}

install_optional_task() {
  if has_cmd task; then
    ok "task already installed."
    return 0
  fi

  local os_distro os pm
  os_distro="$(detect_os)"
  os="${os_distro%%|*}"
  pm="$(detect_pkg_mgr "$os")"

  info "Installing optional tool: task ..."

  case "$os:$pm" in
    macos:brew)
      brew install go-task/tap/go-task
      ;;
    linux:apt|linux:dnf|linux:yum|linux:zypper|linux:pacman|linux:none|macos:port|macos:none)
      if has_cmd go; then
        GO111MODULE=on go install github.com/go-task/task/v3/cmd/task@latest
      else
        warn "Go is not installed; skipping task installation."
        return 0
      fi
      ;;
    *)
      warn "No supported automatic installer for task on this system."
      return 0
      ;;
  esac

  if has_cmd task || [[ -x "$HOME/go/bin/task" ]]; then
    ok "task installed."
  else
    warn "task installation failed or is not on PATH yet."
  fi
}

install_docker() {
  local os="$1"
  local pm="$2"
  local s
  s="$(need_sudo)"

  if has_cmd docker; then
    ok "Docker already installed."
    return
  fi

  info "Installing Docker ..."
  if [ "$os" = "macos" ]; then
    case "$pm" in
      brew)
        brew install --cask docker
        ok "Docker Desktop installed. Launch Docker.app once to finish setup."
        ;;
      *)
        warn "Homebrew not found. Install Docker Desktop manually."
        ;;
    esac
    return
  fi

  case "$pm" in
    apt)
      $s apt-get update -y
      $s apt-get install -y docker.io docker-compose-plugin
      $s systemctl enable --now docker
      ;;
    dnf)
      $s dnf install -y docker docker-compose
      $s systemctl enable --now docker
      ;;
    yum)
      $s yum install -y docker docker-compose
      $s systemctl enable --now docker
      ;;
    pacman)
      $s pacman -Sy --noconfirm docker docker-compose
      $s systemctl enable --now docker
      ;;
    zypper)
      $s zypper --non-interactive install docker docker-compose
      $s systemctl enable --now docker
      ;;
    *)
      err "Unsupported package manager for Docker. Please install manually."
      exit 1
      ;;
  esac

  ok "Docker installed."
}

install_podman() {
  local os="$1"
  local pm="$2"
  local s
  s="$(need_sudo)"

  if has_cmd podman; then
    ok "Podman already installed."
    return
  fi

  info "Installing Podman ..."
  if [ "$os" = "macos" ]; then
    case "$pm" in
      brew)
        brew install podman
        ok "Podman CLI installed."
        warn "Run: podman machine init && podman machine start"
        ;;
      *)
        warn "Homebrew not found. Install Podman manually."
        ;;
    esac
    return
  fi

  case "$pm" in
    apt)    $s apt-get update -y && $s apt-get install -y podman ;;
    dnf)    $s dnf install -y podman ;;
    yum)    $s yum install -y podman ;;
    pacman) $s pacman -Sy --noconfirm podman ;;
    zypper) $s zypper --non-interactive install podman ;;
    *)
      err "Unsupported package manager for Podman. Please install manually."
      exit 1
      ;;
  esac

  ok "Podman installed."
}

ensure_dev_tools() {
  local pkg_mgr="$1"
  install_rustup "$pkg_mgr"

  ensure_rust_component rustfmt
  ensure_rust_component clippy
  ensure_rust_component llvm-tools-preview
  install_cargo_binary cargo-llvm-cov cargo-llvm-cov
  install_cargo_binary cargo-deny cargo-deny

  install_mise
  install_lefthook
}

install_optional_tools() {
    if ! has_cmd just; then
        read -p "Do you want to install 'just'? (y/n): " response
        if [[ "$response" =~ ^[Yy]$ ]]; then
            install_optional_just
        else
            info "'just' installation skipped."
        fi
    fi

    if ! has_cmd task; then
        read -p "Do you want to install 'task'? (y/n): " response
        if [[ "$response" =~ ^[Yy]$ ]]; then
            install_optional_task
        else
            info "'task' installation skipped."
        fi
    fi
}

install_container_engine() {
  local os="$1"
  local pkg_mgr="$2"

  if has_cmd docker || has_cmd podman; then
    ok "Container runtime already available."
    has_cmd docker && ok "docker found"
    has_cmd podman && ok "podman found"
    return 0
  fi

  echo
  warn "No container runtime detected (docker or podman)."

  printf "Install a container engine? [y/N]: "
  local response
  read -r response || true

  if [[ ! "$response" =~ ^[Yy]$ ]]; then
    info "Skipping container engine installation."
    return 0
  fi

  echo "Select container engine to install:"
  echo "  1) Docker"
  echo "  2) Podman"
  echo "  3) Both"
  printf "Enter choice [1-3]: "

  local choice
  read -r choice || true

  case "$choice" in
    1)
      install_docker "$os" "$pkg_mgr"
      ;;
    2)
      install_podman "$os" "$pkg_mgr"
      ;;
    3)
      install_docker "$os" "$pkg_mgr"
      install_podman "$os" "$pkg_mgr"
      ;;
    *)
      warn "Invalid choice, skipping container installation."
      ;;
  esac
}