#!/usr/bin/env bash
set -euo pipefail

REPO="dextryayers/klyron"
BIN_NAME="klyron"
VERSION="${VERSION:-latest}"

# ── Colors ──────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; CYAN='\033[0;36m'; NC='\033[0m'

# ── Platform detection ──────────────────────────────────────────

detect_arch() {
  local arch
  arch="$(uname -m)"
  case "$arch" in
    x86_64|amd64) echo "x86_64" ;;
    aarch64|arm64) echo "aarch64" ;;
    *) echo -e "${RED}Unsupported architecture: $arch${NC}" >&2; exit 1 ;;
  esac
}

detect_os() {
  local os
  os="$(uname -s)"
  case "$os" in
    Linux)  echo "linux" ;;
    Darwin) echo "macos" ;;
    CYGWIN*|MINGW*|MSYS*) echo "windows" ;;
    *) echo -e "${RED}Unsupported OS: $os${NC}" >&2; exit 1 ;;
  esac
}

# ── Download & prepare binary ───────────────────────────────────

find_local_binary() {
  local os="$1"
  local ext=""
  [ "$os" = "windows" ] && ext=".exe"
  local dirs
  dirs="$PWD/target/release $PWD/target/debug $PWD/bin $PWD"
  for d in $dirs; do
    local candidate="${d}/${BIN_NAME}${ext}"
    if [ -f "$candidate" ] && [ -x "$candidate" ]; then
      echo "$candidate"
      return 0
    fi
  done
  return 1
}

download_and_extract() {
  local os="$1" arch="$2"
  local url bin_path

  # Try local binary first (development / local build)
  local local_bin
  local_bin=$(find_local_binary "$os") || true
  if [ -n "$local_bin" ]; then
    echo -e "${CYAN}Using local build: ${local_bin}${NC}" >&2
    local dest="/tmp/${BIN_NAME}"
    [ "$os" = "windows" ] && dest="/tmp/${BIN_NAME}.exe"
    cp "$local_bin" "$dest"
    chmod +x "$dest"
    echo "$dest"
    return 0
  fi

  if [ "$os" = "windows" ]; then
    if [ "$VERSION" = "latest" ]; then
      url="https://github.com/$REPO/releases/latest/download/${BIN_NAME}-windows-${arch}.zip"
    else
      url="https://github.com/$REPO/releases/download/${VERSION}/${BIN_NAME}-windows-${arch}.zip"
    fi
    echo -e "${CYAN}Downloading ${BIN_NAME} for windows/${arch} ...${NC}" >&2
    curl -fsSL "$url" -o "/tmp/${BIN_NAME}.zip" || {
      echo -e "${RED}Download failed (HTTP 404). No release found for ${VERSION}.${NC}" >&2
      echo "Either build from source with 'cargo build' or install via 'npm install -g klyron'" >&2
      exit 1
    }
    echo -e "${CYAN}Extracting ...${NC}" >&2
    local tmpdir
    tmpdir=$(mktemp -d "/tmp/${BIN_NAME}.XXXXX")
    unzip -o "/tmp/${BIN_NAME}.zip" -d "$tmpdir" > /dev/null 2>&1
    rm -f "/tmp/${BIN_NAME}.zip"
    bin_path=$(find "$tmpdir" -name "${BIN_NAME}.exe" -type f 2>/dev/null | head -1)
    if [ -z "$bin_path" ]; then
      echo -e "${RED}Binary not found in archive${NC}" >&2
      rm -rf "$tmpdir"
      exit 1
    fi
    mv "$bin_path" "/tmp/${BIN_NAME}.exe"
    rm -rf "$tmpdir"
    echo "/tmp/${BIN_NAME}.exe"
  else
    if [ "$VERSION" = "latest" ]; then
      url="https://github.com/$REPO/releases/latest/download/${BIN_NAME}-${os}-${arch}.tar.gz"
    else
      url="https://github.com/$REPO/releases/download/${VERSION}/${BIN_NAME}-${os}-${arch}.tar.gz"
    fi
    echo -e "${CYAN}Downloading ${BIN_NAME} for ${os}/${arch} ...${NC}" >&2
    curl -fsSL "$url" -o "/tmp/${BIN_NAME}.tar.gz" || {
      echo -e "${RED}Download failed (HTTP 404). No release found for ${VERSION}.${NC}" >&2
      echo "Either build from source with 'cargo build' or install via 'npm install -g klyron'" >&2
      exit 1
    }
    echo -e "${CYAN}Extracting ...${NC}" >&2
    tar xzf "/tmp/${BIN_NAME}.tar.gz" -C "/tmp/"
    rm -f "/tmp/${BIN_NAME}.tar.gz"
    if [ ! -f "/tmp/${BIN_NAME}" ]; then
      local found
      found=$(find /tmp -name "${BIN_NAME}" -type f 2>/dev/null | head -1)
      if [ -n "$found" ]; then
        mv "$found" "/tmp/${BIN_NAME}"
      else
        echo -e "${RED}Binary not found in archive${NC}" >&2
        exit 1
      fi
    fi
    chmod +x "/tmp/${BIN_NAME}"
    echo "/tmp/${BIN_NAME}"
  fi
}

# ── Install to destination ──────────────────────────────────────

install_binary() {
  local src="$1" dest_dir="$2"
  local dest="${dest_dir}/${BIN_NAME}"
  case "$(detect_os)" in
    windows) dest="${dest_dir}/${BIN_NAME}.exe" ;;
  esac

  # Try direct copy
  mkdir -p "$dest_dir" 2>/dev/null || true
  if cp "$src" "$dest" 2>/dev/null; then
    chmod +x "$dest"
    rm -f "$src"
    echo -e "${GREEN}Installed ${BIN_NAME} to ${dest}${NC}"
    return 0
  fi

  # Try with sudo
  if command -v sudo &>/dev/null; then
    echo -e "${CYAN}Writing to ${dest_dir} requires sudo ...${NC}"
    sudo mkdir -p "$dest_dir"
    sudo cp "$src" "$dest"
    sudo chmod +x "$dest"
    rm -f "$src"
    echo -e "${GREEN}Installed ${BIN_NAME} to ${dest}${NC}"
    return 0
  fi

  echo -e "${RED}Cannot write to ${dest_dir}. Please run with sudo or set INSTALL_DIR to a writable path.${NC}" >&2
  echo "Binary is at: $src"
  echo "Install manually: sudo cp $src $dest"
  exit 1
}

# ── PATH check ──────────────────────────────────────────────────

check_path() {
  local dest_dir="$1"
  if command -v "$BIN_NAME" &>/dev/null; then
    return 0
  fi
  case ":$PATH:" in
    *":$dest_dir:"*) return 0 ;;
  esac
  echo -e "${CYAN}Note: ${dest_dir} is not in your PATH.${NC}"
  echo "  Add it by running:"
  echo "    export PATH=\"\$PATH:$dest_dir\""
  echo "  Or add that line to your ~/.bashrc / ~/.zshrc"
}

# ── Main ────────────────────────────────────────────────────────

main() {
  local os arch dest

  os="$(detect_os)"
  arch="$(detect_arch)"

  # Default: /usr/local/bin for Unix, /usr/local/bin for MSYS2/Git Bash
  case "$os" in
    linux|macos) dest="${INSTALL_DIR:-/usr/local/bin}" ;;
    windows)     dest="${INSTALL_DIR:-/usr/local/bin}" ;;
  esac
  dest="${INSTALL_DIR:-$dest}"

  echo -e "${CYAN}Installing Klyron v${VERSION} for ${os}/${arch}${NC}"
  echo ""

  local bin_src
  bin_src=$(download_and_extract "$os" "$arch")
  install_binary "$bin_src" "$dest"

  echo ""
  if command -v "$BIN_NAME" &>/dev/null; then
    echo -e "${GREEN}Klyron installed successfully!${NC}"
    "$BIN_NAME" --version
  else
    check_path "$dest"
    echo ""
    echo -e "${GREEN}Klyron installed to ${dest}/${BIN_NAME}${NC}"
    echo "  Run '${dest}/${BIN_NAME} --version' to verify."
  fi
}

main