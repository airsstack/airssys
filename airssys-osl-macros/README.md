# airssys-osl-macros

Procedural macros for [airssys-osl](../airssys-osl) core abstractions.

## Overview

This crate provides ergonomic macros to reduce boilerplate when implementing
airssys-osl traits. The `#[executor]` macro automatically generates `OSExecutor<O>` 
trait implementations, eliminating ~85% of repetitive code.

## Status

**Phase 2 Complete** - Core `#[executor]` macro fully implemented and tested.

- ✅ 27 unit tests passing
- ✅ Full operation mapping (11 operations)
- ✅ Code generation working
- ✅ Zero warnings, production ready
- ⏳ Integration with airssys-osl (Phase 3)

See [progress tracking](../.copilot/memory_bank/sub_projects/airssys-osl-macros/progress.md) for detailed status.

## Available Macros

### `#[executor]` - **Implemented** ✅

Automatically generates `OSExecutor<O>` trait implementations from method names.

**Before** (without macro):
```rust
use airssys_osl::prelude::*;

struct MyExecutor;

impl MyExecutor {
    async fn file_read(&self, operation: FileReadOperation, context: &ExecutionContext)
        -> OSResult<ExecutionResult> {
        // Your implementation
        todo!()
    }
}

#[async_trait::async_trait]
impl OSExecutor<FileReadOperation> for MyExecutor {
    async fn execute(&self, operation: FileReadOperation, context: &ExecutionContext)
        -> OSResult<ExecutionResult> {
        self.file_read(operation, context).await
    }
}
```

**After** (with macro):
```rust
use airssys_osl::prelude::*;
use airssys_osl_macros::executor;

struct MyExecutor;

#[executor]
impl MyExecutor {
    async fn file_read(&self, operation: FileReadOperation, context: &ExecutionContext)
        -> OSResult<ExecutionResult> {
        // Your implementation
        todo!()
    }
}
// That's it! The trait implementation is generated automatically.
```

### Multiple Operations

The macro supports multiple operations in a single impl block:

```rust
#[executor]
impl MyExecutor {
    async fn file_read(&self, operation: FileReadOperation, context: &ExecutionContext)
        -> OSResult<ExecutionResult> { todo!() }
    
    async fn file_write(&self, operation: FileWriteOperation, context: &ExecutionContext)
        -> OSResult<ExecutionResult> { todo!() }
    
    async fn process_spawn(&self, operation: ProcessSpawnOperation, context: &ExecutionContext)
        -> OSResult<ExecutionResult> { todo!() }
}
// Generates 3 separate OSExecutor<O> trait implementations
```

### Supported Operations

The macro recognizes 11 operation methods:

**Filesystem (5):**
- `file_read` → `FileReadOperation`
- `file_write` → `FileWriteOperation`
- `file_delete` → `FileDeleteOperation`
- `directory_create` → `DirectoryCreateOperation`
- `directory_list` → `DirectoryListOperation`

**Process (3):**
- `process_spawn` → `ProcessSpawnOperation`
- `process_kill` → `ProcessKillOperation`
- `process_signal` → `ProcessSignalOperation`

**Network (3):**
- `network_connect` → `NetworkConnectOperation`
- `network_listen` → `NetworkListenOperation`
- `network_socket` → `NetworkSocketOperation`

### Helper Methods

Non-operation methods are preserved and ignored by the macro:

```rust
#[executor]
impl MyExecutor {
    async fn file_read(&self, operation: FileReadOperation, context: &ExecutionContext)
        -> OSResult<ExecutionResult> {
        self.validate_path(&operation.path)?;
        todo!()
    }
    
    // Helper method - ignored by macro
    fn validate_path(&self, path: &str) -> OSResult<()> {
        Ok(())
    }
}
```

### Method Signature Requirements

Each operation method must follow this signature:

```rust
async fn operation_name(
    &self,                      // Must be &self (not &mut self or self)
    operation: OperationType,   // Parameter must be named "operation"
    context: &ExecutionContext  // Parameter must be named "context"
) -> OSResult<ExecutionResult>  // Return type
```

### Error Messages

The macro provides helpful error messages:

- "Method 'file_read' must be async"
- "Method must take &self (not &mut self or self)"
- "First parameter must be named 'operation', found 'op'"
- "Duplicate operation method 'file_read'"

## Future Macros (Planned)

- `#[operation]` - Derive Operation trait
- `#[middleware]` - Generate Middleware<O> implementations

## Development

This is a proc-macro crate and must be compiled for the host platform.

### Building

```bash
cargo build --package airssys-osl-macros
```

### Testing

```bash
# Run all tests (27 unit tests)
cargo test --package airssys-osl-macros

# Run with coverage
cargo tarpaulin --package airssys-osl-macros
```

### Code Quality

```bash
# Check code
cargo check --package airssys-osl-macros

# Run clippy
cargo clippy --package airssys-osl-macros --all-targets --all-features

# Generate documentation
cargo doc --package airssys-osl-macros --no-deps --open
```

## Documentation

- **Crate docs**: Run `cargo doc --package airssys-osl-macros --open`
- **Integration guide**: See [airssys-osl documentation](../airssys-osl)
- **Examples**: See `examples/` directory in airssys-osl

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../LICENSE-MIT))

at your option.

