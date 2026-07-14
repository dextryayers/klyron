#!/usr/bin/env bash
set -euo pipefail
echo "Benchmarking klyron_crypto bindings..."
(cd "$(dirname "$0")/../.." && cargo bench -p klyron_crypto 2>/dev/null || echo "SKIP")
echo "Done benchmarking klyron_crypto bindings."
