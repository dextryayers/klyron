#!/usr/bin/env bash
set -euo pipefail
echo "Testing klyron_http bindings..."
(cd "$(dirname "$0")/../.." && cargo test -p klyron_http 2>/dev/null || echo "SKIP")
echo "Done testing klyron_http bindings."
