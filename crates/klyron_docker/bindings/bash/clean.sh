#!/usr/bin/env bash
set -euo pipefail
echo "Cleaning klyron_docker bindings..."
rm -rf build/ dist/ node_modules/
echo "[docker] clean complete"
