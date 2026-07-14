#!/usr/bin/env bash
set -euo pipefail
echo "Benchmarking klyron_loader bindings..."
(cd "$(dirname "$0")/../.." && cargo bench -p klyron_loader 2>/dev/null || echo "SKIP")
echo "Done benchmarking klyron_loader bindings."
