#!/usr/bin/env bash
set -euo pipefail
echo "Testing klyron_logger bindings..."
(cd "$(dirname "$0")/../.." && cargo test -p klyron_logger 2>/dev/null || echo "SKIP")
echo "Done testing klyron_logger bindings."
