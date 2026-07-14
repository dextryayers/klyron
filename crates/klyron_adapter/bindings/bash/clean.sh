#!/usr/bin/env bash
set -euo pipefail
echo "Cleaning klyron_adapter bindings..."
rm -rf build/ dist/ node_modules/
echo "[adapter] clean complete"
