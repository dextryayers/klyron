#!/usr/bin/env bash
set -euo pipefail
echo "Cleaning klyron_telemetry bindings..."
rm -rf build/ dist/ node_modules/
echo "[telemetry] clean complete"
