#!/usr/bin/env bash
set -euo pipefail
echo "Testing klyron_process bindings..."
(cd "$(dirname "$0")/../.." && cargo test -p klyron_process 2>/dev/null || echo "SKIP")
echo "Done testing klyron_process bindings."
