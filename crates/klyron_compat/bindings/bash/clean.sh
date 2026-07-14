#!/usr/bin/env bash
set -euo pipefail
echo "Cleaning klyron_compat bindings..."
rm -rf build/ dist/ node_modules/
echo "[compat] clean complete"
