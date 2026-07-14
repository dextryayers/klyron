#!/usr/bin/env bash
set -euo pipefail
echo "Benchmarking klyron_process bindings..."
(cd "$(dirname "$0")/../.." && cargo bench -p klyron_process 2>/dev/null || echo "SKIP")
echo "Done benchmarking klyron_process bindings."
