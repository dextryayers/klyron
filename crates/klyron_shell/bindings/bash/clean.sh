#!/usr/bin/env bash
set -euo pipefail
echo "Cleaning klyron_shell bindings..."
rm -rf build/ dist/ node_modules/
echo "[shell] clean complete"
