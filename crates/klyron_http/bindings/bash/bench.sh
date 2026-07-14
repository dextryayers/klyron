#!/usr/bin/env bash
set -euo pipefail
echo "Benchmarking klyron_http bindings..."
(cd "$(dirname "$0")/../.." && cargo bench -p klyron_http 2>/dev/null || echo "SKIP")
echo "Done benchmarking klyron_http bindings."
