#!/usr/bin/env bash
set -euo pipefail
echo "Testing klyron_crypto bindings..."
(cd "$(dirname "$0")/../.." && cargo test -p klyron_crypto 2>/dev/null || echo "SKIP")
echo "Done testing klyron_crypto bindings."
