#!/bin/bash
set -euo pipefail

echo "Running post-generation setup for {{ name }}..."

cd "{{ project_dir }}"

if command -v pnpm &> /dev/null; then
  pnpm install
elif command -v yarn &> /dev/null; then
  yarn install
else
  npm install
fi

echo "{{ name }} scaffold generated successfully!"
