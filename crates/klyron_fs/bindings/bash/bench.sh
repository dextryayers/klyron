#!/usr/bin/env bash
set -euo pipefail
echo "Benchmarking klyron_fs bindings..."
(cd "$(dirname "$0")/../.." && cargo bench -p klyron_fs 2>/dev/null || echo "SKIP")
echo "Done benchmarking klyron_fs bindings."
