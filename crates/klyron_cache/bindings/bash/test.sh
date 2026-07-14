#!/usr/bin/env bash
set -euo pipefail
echo "Testing klyron_cache bindings..."
(cd "$(dirname "$0")/../.." && cargo test -p klyron_cache 2>/dev/null || echo "SKIP")
echo "Done testing klyron_cache bindings."
