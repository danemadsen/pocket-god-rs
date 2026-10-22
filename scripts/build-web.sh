#!/usr/bin/env bash
# Builds the web version: web/index.html loads the wasm-bindgen output in web/pkg/.
#
#   scripts/build-web.sh           build
#   scripts/build-web.sh --serve   build, then serve web/ on http://localhost:8080 (PORT=…)
#
# Needs the wasm32-unknown-unknown rustup target and wasm-bindgen-cli at the version of the
# wasm-bindgen crate in Cargo.lock.
set -euo pipefail

cd "$(dirname "$0")/.."

WASM=target/wasm32-unknown-unknown/release/pocket-god.wasm
OUT_DIR=web/pkg
PORT="${PORT:-8080}"

locked_version=$(grep -A1 '^name = "wasm-bindgen"$' Cargo.lock | sed -n 's/^version = "\(.*\)"$/\1/p')
installed_version=$(wasm-bindgen --version | cut -d' ' -f2)
if [[ "$installed_version" != "$locked_version" ]]; then
    echo "wasm-bindgen $installed_version is installed but Cargo.lock uses $locked_version:" >&2
    echo "  cargo install wasm-bindgen-cli --version $locked_version" >&2
    exit 1
fi

cargo build --locked --release --target wasm32-unknown-unknown -p pocket-god

# The name section (function names for wasm stack traces) is about a third of the module;
# panic messages are still logged without it.
wasm-bindgen --target web --no-typescript --remove-name-section --out-dir "$OUT_DIR" --out-name pocket_god "$WASM"

wasm_megabytes=$(( $(wc -c < "$OUT_DIR/pocket_god_bg.wasm") / 1000000 ))
echo "Built $OUT_DIR/pocket_god_bg.wasm ($wasm_megabytes MB)"

if [[ "${1:-}" == "--serve" ]]; then
    echo "Serving web/ on http://localhost:$PORT"
    python3 -m http.server "$PORT" --directory web
fi
