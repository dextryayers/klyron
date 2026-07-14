#!/usr/bin/env bash
set -euo pipefail

CRATE_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$CRATE_DIR"

echo "==> Running benchmarks..."
cargo bench 2>&1 || cargo test -- --bench 2>&1 || echo "No benchmarks found"
echo "==> Benchmarks complete"
