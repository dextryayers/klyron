#!/usr/bin/env bash
set -euo pipefail
echo "Cleaning klyron_config bindings..."
rm -rf build/ dist/ node_modules/
echo "[config] clean complete"
