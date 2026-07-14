#!/usr/bin/env bash
set -euo pipefail

CRATE_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$CRATE_DIR"

echo "==> Running tests..."
cargo test 2>&1
echo "==> Tests complete"
