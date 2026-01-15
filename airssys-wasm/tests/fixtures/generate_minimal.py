#!/usr/bin/env python3
"""Generate a minimal WASM component for testing."""


def create_minimal_component():
    """Create minimal valid WASM component using inline WAT format."""
    # Minimal component in WAT format
    wat_source = "(component)"

    # Convert to bytes (this is the parsed binary format)
    # We'll use Python to generate the binary directly
    # WASM component format starts with magic number: 0x00 0x61 0x73 0x6D
    # Followed by version: 0x0a 0x00 0x00 0x00 (component model version)

    wasm = bytearray(
        [
            0x00,
            0x61,
            0x73,
            0x6D,  # Magic number (\0asm)
            0x0A,
            0x00,
            0x00,
            0x00,  # Component model version (10)
        ]
    )

    # Write to file
    output_path = "minimal-component.wasm"
    with open(output_path, "wb") as f:
        f.write(wasm)

    print(f"Generated {output_path} ({len(wasm)} bytes)")


if __name__ == "__main__":
    create_minimal_component()
