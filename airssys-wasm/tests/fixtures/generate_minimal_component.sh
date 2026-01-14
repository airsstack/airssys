#!/bin/bash
# Script to generate a minimal WASM component for testing

# We'll use a Python script to create a minimal valid WASM component
# This is needed because we don't have a Rust toolchain set up for this

cat > /tmp/minimal_wit.wit << 'WITEOF'
package test:empty-component@0.1.0

interface empty {
}

world empty-component {
  import empty: interface
}
WITEOF

echo "WIT file created at /tmp/minimal_wit.wit"

# Try using wasm-tools component new with a simple core module
# First, create a minimal core WASM module
python3 << 'PYTHONEOF'
import struct

# Create minimal valid WASM module
# Magic number
wasm = bytearray([0x00, 0x61, 0x73, 0x6D])

# Version
wasm.extend(struct.pack('<I', 1))

# Section 1: Type section (just one function type)
type_section = bytearray([1])  # Section ID
type_section.extend(struct.pack('<B', len([0x60])))  # One type
type_section.extend([0x60, 0x00, 0x00])  # func() -> func()
wasm.extend([len(type_section)])
wasm.extend(type_section)

# Section 2: Function section (no functions)
func_section = bytearray([2])  # Section ID
func_section.extend(struct.pack('<B', 0))  # Zero functions
wasm.extend([len(func_section)])
wasm.extend(func_section)

# Section 7: Export section (no exports)
export_section = bytearray([7])  # Section ID
export_section.extend(struct.pack('<B', 0))  # Zero exports
wasm.extend([len(export_section)])
wasm.extend(export_section)

# Write to file
with open('/tmp/minimal_core.wasm', 'wb') as f:
    f.write(wasm)

print("Core WASM module created")
PYTHONEOF

echo "Core WASM module created at /tmp/minimal_core.wasm"
