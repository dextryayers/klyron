#!/usr/bin/env bash
set -euo pipefail

echo "Starting klyron_updater dev mode..."

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$SCRIPT_DIR/../.."

cd "$CRATE_DIR"
cargo watch -x check -x test 2>&1
