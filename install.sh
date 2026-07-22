#!/usr/bin/env bash
set -euo pipefail

BIN_NAME="klyron"
CRATE_NAME="klyron_cli"

RED='\033[0;31m'; GREEN='\033[0;32m'; CYAN='\033[0;36m'; NC='\033[0m'

usage() {
  echo "Usage: $0 [--release|--debug] [--dir /usr/local/bin]"
  echo "  --release   Build in release mode (optimized, slower build)"
  echo "  --debug     Build in debug mode (fast, default)"
  echo "  --dir       Install destination (default: /usr/local/bin)"
  exit 0
}

MODE="debug"
DEST="/usr/local/bin"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --release) MODE="release" ;;
    --debug)   MODE="debug" ;;
    --dir)     DEST="$2"; shift ;;
    --help|-h) usage ;;
    *) echo "Unknown: $1"; usage ;;
  esac
  shift
done

if [ ! -f "Cargo.toml" ] || ! grep -q "klyron_cli" Cargo.toml 2>/dev/null; then
  SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
  cd "$SCRIPT_DIR"
fi

find_binary() {
  local mode="$1"
  local dir="target/$mode"
  [ -f "$dir/$BIN_NAME" ] && { echo "$dir/$BIN_NAME"; return 0; }
  [ -f "$dir/$CRATE_NAME" ] && { echo "$dir/$CRATE_NAME"; return 0; }
  return 1
}

binary_exists() {
  find_binary "$MODE" >/dev/null 2>&1
}

build_binary() {
  local mode="$1"
  local build_flag=""
  [ "$mode" = "release" ] && build_flag="--release"

  echo -e "${CYAN}Building klyron ($mode)...${NC}" >&2

  if ! command -v cargo &>/dev/null; then
    echo -e "${RED}Rust/Cargo not found. Install from https://rustup.rs${NC}" >&2
    exit 1
  fi

  cargo build $build_flag --package "$CRATE_NAME" 1>&2

  local bin_dir="target/$mode"
  if [ -f "$bin_dir/$CRATE_NAME" ] && [ ! -f "$bin_dir/$BIN_NAME" ]; then
    cp "$bin_dir/$CRATE_NAME" "$bin_dir/$BIN_NAME"
  fi

  if [ ! -f "$bin_dir/$BIN_NAME" ]; then
    echo -e "${RED}Build failed: binary not found at $bin_dir/$BIN_NAME${NC}" >&2
    exit 1
  fi

  echo -e "${GREEN}Built: $bin_dir/$BIN_NAME${NC}" >&2
}

install_binary() {
  local src="$1" dest="$2"

  mkdir -p "$(dirname "$dest")" 2>/dev/null || true
  if cp "$src" "$dest" 2>/dev/null; then
    chmod +x "$dest"
    echo -e "${GREEN}Installed: $dest${NC}"
    return 0
  fi

  if command -v sudo &>/dev/null; then
    sudo mkdir -p "$(dirname "$dest")"
    sudo cp "$src" "$dest"
    sudo chmod +x "$dest"
    echo -e "${GREEN}Installed: $dest${NC}"
    return 0
  fi

  echo -e "${RED}Cannot write to $dest${NC}" >&2
  echo "  Try: sudo cp $src $dest"
  exit 1
}

main() {
  local src

  if binary_exists; then
    src="$(find_binary "$MODE")"
    echo -e "${CYAN}Using existing build: $src${NC}" >&2
  else
    build_binary "$MODE"
    src="$(find_binary "$MODE")"
  fi

  install_binary "$src" "${DEST}/${BIN_NAME}"

  echo ""
  if command -v "$BIN_NAME" &>/dev/null; then
    echo -e "${GREEN}Klyron ready!${NC}"
    "$BIN_NAME" --version
  fi
}

main
