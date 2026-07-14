#!/usr/bin/env bash
set -euo pipefail

CRATE_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$CRATE_DIR"

echo "==> Building crate..."
cargo build --release 2>&1
echo "==> Build complete"
