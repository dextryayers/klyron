#!/usr/bin/env bash
set -euo pipefail
echo "Cleaning klyron_template bindings..."
rm -rf build/ dist/ node_modules/
echo "[template] clean complete"
