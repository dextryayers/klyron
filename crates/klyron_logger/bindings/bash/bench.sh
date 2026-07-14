#!/usr/bin/env bash
set -euo pipefail
echo "Benchmarking klyron_logger bindings..."
(cd "$(dirname "$0")/../.." && cargo bench -p klyron_logger 2>/dev/null || echo "SKIP")
echo "Done benchmarking klyron_logger bindings."
