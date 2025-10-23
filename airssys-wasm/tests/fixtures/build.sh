#!/bin/bash
# Compile WAT fixtures to WASM Component Model format
# Requires: wasm-tools (install via: cargo install wasm-tools)

set -e

FIXTURES_DIR="$(dirname "$0")"

echo "Compiling WAT fixtures to WASM..."

for wat_file in "$FIXTURES_DIR"/*.wat; do
    if [ -f "$wat_file" ]; then
        wasm_file="${wat_file%.wat}.wasm"
        echo "  $wat_file -> $wasm_file"
        wasm-tools parse "$wat_file" -o "$wasm_file"
    fi
done

echo "Done."
