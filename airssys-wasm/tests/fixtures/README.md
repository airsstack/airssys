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

### echo.wasm

- **What**: A WASM component that implements `component-lifecycle` and echoes back message payloads
- **Why**: Tests basic message handling, component loading, and the full WASM component pipeline
- **How**: Built from Rust source in `components/echo/`:
  ```bash
  cd tests/fixtures/components
  ./build-fixtures.sh
  ```
- **WIT Interface**: Implements `airssys:core/component-lifecycle` (all 6 exports)
- **Behavior**: `handle-message` returns `Ok(Some(msg.payload))` (echo)
- **Size**: ~24KB
- **Source**: `tests/fixtures/components/echo/src/lib.rs`
- **Used by**: System integration tests (WASM-TASK-053)

### counter.wasm

- **What**: A stateful WASM component that maintains an internal counter, increments it on each message, and returns the count
- **Why**: Tests stateful component behavior, state persistence across multiple handle-message calls, and WASM instance memory preservation
- **How**: Built from Rust source in `components/counter/`:
  ```bash
  cd tests/fixtures/components
  ./build-fixtures.sh
  ```
- **WIT Interface**: Implements `airssys:core/component-lifecycle` (all 6 exports)
- **Behavior**: `handle-message` increments internal counter and returns current count as UTF-8 string payload (e.g., "1", "2", "3")
- **State**: `stateful: true` -- uses `thread_local!` + `Cell<u32>` for in-memory counter
- **Size**: ~25KB (similar to echo.wasm)
- **Source**: `tests/fixtures/components/counter/src/lib.rs`
- **Used by**: System integration tests (WASM-TASK-053)

## Documentation

For each fixture, document:
- **What** the fixture does
- **Why** it exists (what it tests)
- **How** it was generated (build commands, source location)

## Fixture Generation

Fixtures should be generated from real WASM components following WIT interface definitions.
