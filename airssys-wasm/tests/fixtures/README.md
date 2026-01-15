# Test Fixtures

This directory contains WASM component fixtures for integration testing.

## Purpose

Each fixture represents a WASM component used to test specific functionality of airssys-wasm.

## Fixtures

### minimal-component.wasm

- **What**: A minimal valid WASM component with no functionality
- **Why**: Tests basic component loading, instantiation, and lifecycle operations
- **How**: Generated using `wasm-tools parse` from WAT format:
  ```bash
  echo '(component)' > /tmp/minimal.wat
  wasm-tools parse /tmp/minimal.wat -o minimal-component.wasm
  ```
- **Size**: 8 bytes (magic number + version)
- **Used by**: `store-integration-tests.rs`

## Documentation

For each fixture, document:
- **What** the fixture does
- **Why** it exists (what it tests)
- **How** it was generated (build commands, source location)

## Fixture Generation

Fixtures should be generated from real WASM components following WIT interface definitions.
