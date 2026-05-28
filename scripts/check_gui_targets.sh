#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

check_if_target_installed() {
  local target="$1"
  rustup target list --installed | rg -q "^${target}$"
}

check_target() {
  local target="$1"
  if check_if_target_installed "${target}"; then
    echo "Checking GUI for ${target}"
    cargo check --release --target "${target}" --bin treefold-gui
  else
    echo "Skipping ${target} (target not installed)"
  fi
}

check_target "aarch64-apple-darwin"
check_target "x86_64-unknown-linux-gnu"
check_target "x86_64-pc-windows-msvc"
