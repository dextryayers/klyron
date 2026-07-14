#!/usr/bin/env bash
set -euo pipefail

echo "Benchmarking klyron_engine bindings..."

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$SCRIPT_DIR/../.."

cd "$CRATE_DIR"
cargo bench 2>&1

echo "klyron_engine benchmarks complete"
