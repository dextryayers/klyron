#!/bin/bash
set -euo pipefail

echo "Running post-generation setup for {{ name }}..."

cd "{{ project_dir }}"

if command -v bundle &> /dev/null; then
  bundle install
fi

if [ -f "Gemfile" ]; then
  rails db:migrate
fi

echo "{{ name }} scaffold generated successfully!"
