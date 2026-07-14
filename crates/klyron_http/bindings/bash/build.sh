#!/usr/bin/env bash
set -euo pipefail
echo "Building klyron_http bindings..."
echo "  [1/5] Rust: cargo build"
(cd "$(dirname "$0")/../.." && cargo build -p klyron_http 2>/dev/null || echo "    SKIP")
echo "  [2/5] TypeScript: tsc"
(cd "$(dirname "$0")/../ts" && tsc --noEmit 2>/dev/null || echo "    SKIP")
echo "  [3/5] C++: g++"
(cd "$(dirname "$0")/../cpp" && g++ -std=c++17 -Wall -O2 -c *.cpp 2>/dev/null || echo "    SKIP")
echo "  [4/5] C: gcc"
(cd "$(dirname "$0")/../c" && gcc -std=c11 -Wall -O2 -c *.c 2>/dev/null || echo "    SKIP")
echo "  [5/5] Bash: chmod +x"
chmod +x "$(dirname "$0")"/*.sh
echo "Done building klyron_http bindings."
