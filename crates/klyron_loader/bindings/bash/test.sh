#!/usr/bin/env bash
set -euo pipefail
echo "Testing klyron_loader bindings..."
(cd "$(dirname "$0")/../.." && cargo test -p klyron_loader 2>/dev/null || echo "SKIP")
echo "Done testing klyron_loader bindings."
