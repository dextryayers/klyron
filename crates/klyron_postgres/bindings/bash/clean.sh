#!/usr/bin/env bash
set -euo pipefail

echo "Cleaning klyron_postgres bindings..."

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$SCRIPT_DIR/../.."

cd "$CRATE_DIR"
cargo clean 2>&1

echo "klyron_postgres clean complete"
