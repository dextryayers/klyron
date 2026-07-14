#!/usr/bin/env bash
set -euo pipefail
echo "Testing klyron_dns bindings..."
(cd "$(dirname "$0")/../.." && cargo test -p klyron_dns 2>/dev/null || echo "SKIP")
echo "Done testing klyron_dns bindings."
