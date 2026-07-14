#!/usr/bin/env bash
set -euo pipefail

REPO="dextryayers/klyron"
BIN_NAME="klyron"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

detect_arch() {
  local arch
  arch="$(uname -m)"
  case "$arch" in
    x86_64|amd64) echo "x86_64" ;;
    aarch64|arm64) echo "aarch64" ;;
    *) echo "unsupported: $arch"; exit 1 ;;
  esac
}

detect_os() {
  local os
  os="$(uname -s)"
  case "$os" in
    Linux) echo "linux" ;;
    Darwin) echo "macos" ;;
    *) echo "unsupported: $os"; exit 1 ;;
  esac
}

main() {
  local os arch url
  os="$(detect_os)"
  arch="$(detect_arch)"

  if command -v cargo &>/dev/null; then
    echo "Building from source with cargo..."
    cargo install --git "https://github.com/$REPO" --root "$INSTALL_DIR/.."
    echo "Installed $BIN_NAME to $INSTALL_DIR/$BIN_NAME"
    exit 0
  fi

  echo "Cargo not found, downloading pre-built binary..."
  url="https://github.com/$REPO/releases/latest/download/${BIN_NAME}-${os}-${arch}"
  echo "Downloading $url ..."
  curl -fsSL "$url" -o "$INSTALL_DIR/$BIN_NAME"
  chmod +x "$INSTALL_DIR/$BIN_NAME"
  echo "Installed $BIN_NAME to $INSTALL_DIR/$BIN_NAME"
}

main
