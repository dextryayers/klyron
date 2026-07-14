#!/bin/bash
set -euo pipefail

echo "Running post-generation setup for {{ name }}..."

cd "{{ project_dir }}"

if command -v uv &> /dev/null; then
  uv sync
elif command -v poetry &> /dev/null; then
  poetry install
elif command -v pip &> /dev/null; then
  pip install -r requirements.txt
fi

echo "{{ name }} scaffold generated successfully!"
