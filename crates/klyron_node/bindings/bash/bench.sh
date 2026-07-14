#!/usr/bin/env bash
set -euo pipefail
echo "Benchmarking klyron_node bindings..."
(cd "$(dirname "$0")/../.." && cargo bench -p klyron_node 2>/dev/null || echo "SKIP")
echo "Done benchmarking klyron_node bindings."
