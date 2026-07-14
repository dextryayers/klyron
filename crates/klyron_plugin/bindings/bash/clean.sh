#!/usr/bin/env bash
set -euo pipefail
echo "Cleaning klyron_plugin bindings..."
rm -rf build/ dist/ node_modules/
echo "[plugin] clean complete"
