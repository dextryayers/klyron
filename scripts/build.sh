#!/bin/bash
set -euo pipefail
cargo build --release --package klyron_cli
cp target/release/klyron_cli target/release/klyron
echo "Built: target/release/klyron"
