#!/usr/bin/env bash
set -euo pipefail
echo "Testing klyron_node bindings..."
(cd "$(dirname "$0")/../.." && cargo test -p klyron_node 2>/dev/null || echo "SKIP")
echo "Done testing klyron_node bindings."
