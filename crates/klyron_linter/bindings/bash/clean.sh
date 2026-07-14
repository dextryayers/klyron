#!/usr/bin/env bash
set -euo pipefail

CRATE_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$CRATE_DIR"

echo "==> Cleaning crate artifacts..."
cargo clean 2>&1
rm -rf target/ 2>/dev/null || true
echo "==> Clean complete"
