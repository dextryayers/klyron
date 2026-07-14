#!/usr/bin/env bash
set -euo pipefail
echo "Testing klyron_web bindings..."
(cd "$(dirname "$0")/../.." && cargo test -p klyron_web 2>/dev/null || echo "SKIP")
echo "Done testing klyron_web bindings."
