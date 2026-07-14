#!/bin/bash
set -euo pipefail

echo "Running post-generation setup for {{ name }}..."

cd "{{ project_dir }}"

if command -v composer &> /dev/null; then
  composer install
else
  echo "Composer is required. Please install Composer and run: composer install"
fi

if [ -f "package.json" ]; then
  if command -v pnpm &> /dev/null; then
    pnpm install && pnpm run build
  elif command -v yarn &> /dev/null; then
    yarn install && yarn run build
  else
    npm install && npm run build
  fi
fi

php artisan key:generate

echo "{{ name }} scaffold generated successfully!"
