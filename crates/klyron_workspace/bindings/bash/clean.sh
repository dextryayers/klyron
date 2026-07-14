#!/usr/bin/env bash
set -euo pipefail
echo "Cleaning klyron_workspace bindings..."
rm -rf build/ dist/ node_modules/
echo "[workspace] clean complete"
