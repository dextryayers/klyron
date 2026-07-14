#!/usr/bin/env bash
set -euo pipefail
echo "Starting dev mode for klyron_process..."
(cd "$(dirname "$0")/../.." && cargo watch -p klyron_process 2>/dev/null || echo "SKIP: install cargo-watch")
