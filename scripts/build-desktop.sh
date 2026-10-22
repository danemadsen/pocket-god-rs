#!/usr/bin/env bash
# Builds the desktop executable: target/release/pocket-god.
#
#   scripts/build-desktop.sh         build
#   scripts/build-desktop.sh --run   build, then run
set -euo pipefail

cd "$(dirname "$0")/.."

cargo build --locked --release -p pocket-god

echo "Built target/release/pocket-god"

if [[ "${1:-}" == "--run" ]]; then
    exec target/release/pocket-god
fi
