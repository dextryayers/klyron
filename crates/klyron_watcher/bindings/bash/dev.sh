#!/usr/bin/env bash
set -euo pipefail

CRATE_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$CRATE_DIR"

case "${1:-build}" in
    build)
        cargo build
        ;;
    watch)
        cargo watch -x build
        ;;
    test)
        cargo test
        ;;
    doc)
        cargo doc --open
        ;;
    check)
        cargo check
        ;;
    *)
        echo "Usage: $0 {build|watch|test|doc|check}"
        exit 1
        ;;
esac
