#!/bin/bash
set -euo pipefail

echo "Setting up {{ name }}..."

bundle install
rails db:create
rails db:migrate
rails db:seed

echo "{{ name }} setup complete!"
