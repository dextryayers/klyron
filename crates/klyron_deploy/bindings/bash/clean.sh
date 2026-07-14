#!/usr/bin/env bash
set -euo pipefail
echo "Cleaning klyron_deploy bindings..."
rm -rf build/ dist/ node_modules/
echo "[deploy] clean complete"
