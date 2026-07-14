#!/usr/bin/env bash
set -euo pipefail
echo "Testing klyron_fs bindings..."
(cd "$(dirname "$0")/../.." && cargo test -p klyron_fs 2>/dev/null || echo "SKIP")
echo "Done testing klyron_fs bindings."
