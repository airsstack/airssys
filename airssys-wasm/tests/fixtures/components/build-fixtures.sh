#!/bin/bash
# Build WASM component test fixtures
#
# Usage: ./build-fixtures.sh
# Run from: airssys-wasm/tests/fixtures/components/
#
# Prerequisites:
#   - Rust toolchain with wasm32-unknown-unknown target
#   - wasm-tools (https://github.com/bytecodealliance/wasm-tools)
#
# References:
#   - ADR-WASM-032: Test fixture specifications
#   - KNOWLEDGE-WASM-043: Guest-side build process

set -euo pipefail

# --- Prerequisites check ---
command -v wasm-tools >/dev/null 2>&1 || { echo "Error: wasm-tools not found. Install from https://github.com/bytecodealliance/wasm-tools"; exit 1; }
rustup target list --installed | grep -q wasm32-unknown-unknown || { echo "Error: wasm32-unknown-unknown target not installed. Run: rustup target add wasm32-unknown-unknown"; exit 1; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FIXTURES_DIR="$(dirname "$SCRIPT_DIR")"

echo "Building WASM component fixtures..."
echo "Script dir: $SCRIPT_DIR"
echo "Fixtures dir: $FIXTURES_DIR"

# --- echo component ---
echo ""
echo "=== Building echo component ==="
cd "$SCRIPT_DIR/echo"
cargo build --target wasm32-unknown-unknown --release
wasm-tools component new \
  target/wasm32-unknown-unknown/release/echo_component.wasm \
  -o "$FIXTURES_DIR/echo.wasm"
echo "echo.wasm: $(wc -c < "$FIXTURES_DIR/echo.wasm") bytes"
wasm-tools validate "$FIXTURES_DIR/echo.wasm" --features component-model
echo "echo.wasm: VALID"

# --- counter component ---
echo ""
echo "=== Building counter component ==="
cd "$SCRIPT_DIR/counter"
cargo build --target wasm32-unknown-unknown --release
wasm-tools component new \
  target/wasm32-unknown-unknown/release/counter_component.wasm \
  -o "$FIXTURES_DIR/counter.wasm"
echo "counter.wasm: $(wc -c < "$FIXTURES_DIR/counter.wasm") bytes"
wasm-tools validate "$FIXTURES_DIR/counter.wasm" --features component-model
echo "counter.wasm: VALID"

echo ""
echo "All fixtures built successfully."
