#!/usr/bin/env bash
set -euo pipefail
echo "Cleaning klyron_http bindings..."
rm -f "$(dirname "$0")/../cpp/"*.o
rm -f "$(dirname "$0")/../c/"*.o
rm -rf "$(dirname "$0")/../ts/"node_modules
echo "Done cleaning klyron_http bindings."
