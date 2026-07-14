#!/bin/bash
set -euo pipefail

echo "Running post-generation setup for {{ name }}..."

cd "{{ project_dir }}"

if command -v rustup &> /dev/null; then
  echo "Building project..."
  cargo build
fi

echo "{{ name }} scaffold generated successfully!"
