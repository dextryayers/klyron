#!/usr/bin/env bash
set -euo pipefail

BIN_NAME="klyron"
REPO="dextryayers/klyron"

RED='\033[0;31m'; GREEN='\033[0;32m'; CYAN='\033[0;36m'; YELLOW='\033[1;33m'; NC='\033[0m'

warn()  { echo -e "${YELLOW}WARN${NC} $1"; }
info()  { echo -e "${CYAN}INFO${NC} $1"; }
success() { echo -e "${GREEN}OK${NC} $1"; }

detect_os() {
  local os
  os="$(uname -s 2>/dev/null || echo "unknown")"
  case "$os" in
    Linux)  echo "linux" ;;
    Darwin) echo "macos" ;;
    CYGWIN*|MINGW*|MSYS*) echo "windows" ;;
    *) echo "unknown" ;;
  esac
}

remove_binary() {
  local os removed=0
  os="$(detect_os)"

  local paths=(
    "/usr/local/bin/${BIN_NAME}"
    "/usr/bin/${BIN_NAME}"
    "${HOME}/.local/bin/${BIN_NAME}"
    "${HOME}/bin/${BIN_NAME}"
  )
  if [ "$os" = "windows" ]; then
    paths+=("/usr/local/bin/${BIN_NAME}.exe")
    local win_home="${LOCALAPPDATA:-$HOME/AppData/Local}"
    paths+=("${win_home}/klyron/${BIN_NAME}.exe")
  fi
  if [ -n "${INSTALL_DIR:-}" ]; then
    paths+=("${INSTALL_DIR}/${BIN_NAME}")
  fi

  for p in "${paths[@]}"; do
    if [ -f "$p" ] || [ -L "$p" ]; then
      if rm -f "$p" 2>/dev/null; then
        info "Removed $p"; removed=1
      elif command -v sudo &>/dev/null; then
        sudo rm -f "$p" && info "Removed $p (sudo)" && removed=1
      else
        warn "Skipped $p (permission denied). Try: sudo rm $p"
      fi
    fi
  done

  if command -v "$BIN_NAME" &>/dev/null 2>&1; then
    local other; other=$(command -v "$BIN_NAME")
    warn "Binary still on PATH at: $other"
  elif [ "$removed" -eq 1 ]; then
    success "Binary removed from PATH"
  else
    info "No klyron binary found"
  fi
}

remove_config() {
  local count=0
  local dirs=("${HOME}/.klyron" "${HOME}/.config/klyron")
  for d in "${dirs[@]}"; do
    if [ -d "$d" ]; then
      rm -rf "$d" && info "Removed config: $d" && count=$((count+1))
    fi
  done
  [ "$count" -eq 0 ] && info "No config directory found"
}

remove_cache() {
  local count=0
  local dirs=()
  if [ -d "${HOME}/.cache/klyron" ]; then
    dirs+=("${HOME}/.cache/klyron")
  fi
  if [ -d "${HOME}/Library/Caches/klyron" ]; then
    dirs+=("${HOME}/Library/Caches/klyron")
  fi
  if [ -n "${XDG_CACHE_HOME:-}" ] && [ -d "${XDG_CACHE_HOME}/klyron" ]; then
    dirs+=("${XDG_CACHE_HOME}/klyron")
  fi
  for d in "${dirs[@]}"; do
    rm -rf "$d" && info "Removed cache: $d" && count=$((count+1))
  done
  [ "$count" -eq 0 ] && info "No cache directory found"
}

remove_completions() {
  local files=(
    "${HOME}/.local/share/bash-completion/completions/klyron"
    "${HOME}/.zsh/completions/_klyron"
    "${HOME}/.config/fish/completions/klyron.fish"
  )
  for f in "${files[@]}"; do
    if [ -f "$f" ]; then
      rm -f "$f" && info "Removed completions: $f"
    fi
  done
}

remove_npm_global() {
  if ! command -v npm &>/dev/null; then return; fi
  local npm_prefix
  npm_prefix=$(npm prefix -g 2>/dev/null || true)
  if [ -z "$npm_prefix" ] || [ ! -f "${npm_prefix}/lib/node_modules/klyron/package.json" ]; then
    return
  fi
  echo ""
  echo -e "${YELLOW}Klyron is installed as an npm global package at $npm_prefix.${NC}"
  read -rp "Uninstall npm global klyron? [y/N] " yn
  if [[ "$yn" =~ ^[Yy] ]]; then
    npm uninstall -g klyron && success "npm global klyron uninstalled" || warn "npm uninstall failed"
  fi
}

remove_windows_path() {
  [ "$(detect_os)" != "windows" ] && return
  local install_dir="${LOCALAPPDATA:-$HOME/AppData/Local}/klyron"
  echo ""
  if echo "$PATH" | grep -qi "klyron"; then
    info "Klyron found in Windows PATH. To remove it:"
    echo "  System Properties > Advanced > Environment Variables"
    echo "  Or run PowerShell as Admin:"
    echo "    [Environment]::SetEnvironmentVariable('Path',"
    echo "      \$env:Path -replace ';$install_dir', '', 'User')"
  fi
}

echo ""
echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}  Klyron Uninstaller${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""

remove_binary
echo ""
remove_config
echo ""
remove_cache
echo ""
remove_completions
echo ""
remove_npm_global
echo ""
remove_windows_path
echo ""
echo -e "${CYAN}========================================${NC}"
echo -e "${GREEN}Uninstall complete.${NC}"
echo -e "${YELLOW}Note: Project directories and klyron.json files are not removed.${NC}"
echo -e "${CYAN}========================================${NC}"
