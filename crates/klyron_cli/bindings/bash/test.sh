#!/usr/bin/env bash
set -euo pipefail

echo "Testing klyron_cli bindings..."

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$SCRIPT_DIR/../.."

cd "$CRATE_DIR"
cargo test 2>&1

echo "klyron_cli tests complete"
