#!/bin/bash
set -euo pipefail

echo "Running post-generation setup for {{ name }}..."

cd "{{ project_dir }}"

if command -v go &> /dev/null; then
  go mod tidy
  go build
fi

echo "{{ name }} scaffold generated successfully!"
