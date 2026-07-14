#!/usr/bin/env bash
set -euo pipefail
echo "Benchmarking klyron_cache bindings..."
(cd "$(dirname "$0")/../.." && cargo bench -p klyron_cache 2>/dev/null || echo "SKIP")
echo "Done benchmarking klyron_cache bindings."
