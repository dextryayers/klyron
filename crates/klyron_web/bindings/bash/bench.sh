#!/usr/bin/env bash
set -euo pipefail
echo "Benchmarking klyron_web bindings..."
(cd "$(dirname "$0")/../.." && cargo bench -p klyron_web 2>/dev/null || echo "SKIP")
echo "Done benchmarking klyron_web bindings."
