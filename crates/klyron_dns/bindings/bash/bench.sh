#!/usr/bin/env bash
set -euo pipefail
echo "Benchmarking klyron_dns bindings..."
(cd "$(dirname "$0")/../.." && cargo bench -p klyron_dns 2>/dev/null || echo "SKIP")
echo "Done benchmarking klyron_dns bindings."
