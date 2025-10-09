# Macros API Reference

This page provides the API reference for procedural macros available in the `airssys-osl-macros` crate.

## Overview

The `airssys-osl-macros` crate provides procedural macros that simplify the development of custom OS executors by automatically generating boilerplate implementation code.

## Feature Flag

To use macros in your project, enable the `macros` feature:

```toml
[dependencies]
airssys-osl = { version = "0.1", features = ["macros"] }
```

## Available Macros

### `#[executor]`

The `#[executor]` attribute macro automatically implements the `OSExecutor` trait for custom executor types.

#### Basic Usage

```rust
use airssys_osl::prelude::*;

#[executor]
struct MyExecutor;
```

This generates implementations for all 11 supported operations across 3 operation domains.

#### Syntax

```rust
#[executor]
#[executor(name = "CustomName")]
#[executor(operations = [Domain1, Domain2])]
#[executor(name = "CustomName", operations = [Domain1, Domain2])]
```

#### Attributes

| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | String literal | No | Custom executor name for logging/debugging |
| `operations` | Array of operation domains | No | Limit to specific operation domains |

#### Operation Domains

The macro supports these operation domains:

| Domain | Operations | Count |
|--------|-----------|-------|
| `Filesystem` | `FileRead`, `FileWrite`, `FileDelete`, `DirectoryCreate`, `DirectoryList` | 5 |
| `Process` | `ProcessSpawn`, `ProcessKill`, `ProcessSignal` | 3 |
| `Network` | `NetworkConnect`, `NetworkListen`, `NetworkSocket` | 3 |

**Total**: 11 operations across 3 domains

#### Method Signature Requirements

For each operation you want to handle, implement a method with this exact signature:

```rust
async fn execute_{operation}(
    &self,
    operation: {OperationType},
    context: ExecutionContext,
) -> Result<{ResultType}, ExecutionError>
```

**Requirements**:
- Method must be `async`
- Parameter names must be exactly `operation` and `context`
- Return type must be `Result<{ResultType}, ExecutionError>`
- Result type must match the operation's expected output

##### Operation Method Mapping

| Operation | Method Name | Operation Type | Result Type |
|-----------|------------|----------------|-------------|
| `FileRead` | `execute_file_read` | `FileReadOperation` | `FileReadResult` |
| `FileWrite` | `execute_file_write` | `FileWriteOperation` | `FileWriteResult` |
| `FileDelete` | `execute_file_delete` | `FileDeleteOperation` | `FileDeleteResult` |
| `DirectoryCreate` | `execute_directory_create` | `DirectoryCreateOperation` | `DirectoryCreateResult` |
| `DirectoryList` | `execute_directory_list` | `DirectoryListOperation` | `DirectoryListResult` |
| `ProcessSpawn` | `execute_process_spawn` | `ProcessSpawnOperation` | `ProcessSpawnResult` |
| `ProcessKill` | `execute_process_kill` | `ProcessKillOperation` | `ProcessKillResult` |
| `ProcessSignal` | `execute_process_signal` | `ProcessSignalOperation` | `ProcessSignalResult` |
| `NetworkConnect` | `execute_network_connect` | `NetworkConnectOperation` | `NetworkConnectResult` |
| `NetworkListen` | `execute_network_listen` | `NetworkListenOperation` | `NetworkListenResult` |
| `NetworkSocket` | `execute_network_socket` | `NetworkSocketOperation` | `NetworkSocketResult` |

#### Examples

##### Single Domain Executor

```rust
use airssys_osl::prelude::*;

#[executor(operations = [Filesystem])]
struct FileSystemExecutor;

impl FileSystemExecutor {
    async fn execute_file_read(
        &self,
        operation: FileReadOperation,
        _context: ExecutionContext,
    ) -> Result<FileReadResult, ExecutionError> {
        // Custom implementation
        Ok(FileReadResult {
            content: vec![],
            bytes_read: 0,
            completed_at: Utc::now(),
        })
    }

    async fn execute_file_write(
        &self,
        operation: FileWriteOperation,
        _context: ExecutionContext,
    ) -> Result<FileWriteResult, ExecutionError> {
        // Custom implementation
        Ok(FileWriteResult {
            bytes_written: operation.content.len(),
            completed_at: Utc::now(),
        })
    }
}
```

##### Multi-Domain Executor

```rust
use airssys_osl::prelude::*;

#[executor(
    name = "MultiDomainExecutor",
    operations = [Filesystem, Process, Network]
)]
struct MyExecutor;

impl MyExecutor {
    // Implement required methods for Filesystem operations
    async fn execute_file_read(
        &self,
        operation: FileReadOperation,
        context: ExecutionContext,
    ) -> Result<FileReadResult, ExecutionError> {
        // Implementation
    }

    // Implement required methods for Process operations
    async fn execute_process_spawn(
        &self,
        operation: ProcessSpawnOperation,
        context: ExecutionContext,
    ) -> Result<ProcessSpawnResult, ExecutionError> {
        // Implementation
    }

    // Implement required methods for Network operations
    async fn execute_network_connect(
        &self,
        operation: NetworkConnectOperation,
        context: ExecutionContext,
    ) -> Result<NetworkConnectResult, ExecutionError> {
        // Implementation
    }

    // ... other required methods
}
```

##### Executor with Helper Methods

```rust
use airssys_osl::prelude::*;
use std::collections::HashMap;

#[executor(operations = [Filesystem])]
struct CachedExecutor {
    cache: HashMap<String, Vec<u8>>,
}

impl CachedExecutor {
    // Helper methods are preserved
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    fn get_from_cache(&self, path: &str) -> Option<&Vec<u8>> {
        self.cache.get(path)
    }

    async fn execute_file_read(
        &self,
        operation: FileReadOperation,
        _context: ExecutionContext,
    ) -> Result<FileReadResult, ExecutionError> {
        // Use helper method
        if let Some(cached) = self.get_from_cache(&operation.path) {
            return Ok(FileReadResult {
                content: cached.clone(),
                bytes_read: cached.len(),
                completed_at: Utc::now(),
            });
        }
        // ... fallback logic
    }
}
```

## Error Messages

The macro provides helpful error messages for common mistakes:

### Missing Method Implementation

```text
error: Missing required method for operation: execute_file_read
  --> src/lib.rs:5:1
   |
5  | #[executor]
   | ^^^^^^^^^^^
   |
   = note: Required signature:
           async fn execute_file_read(
               &self,
               operation: FileReadOperation,
               context: ExecutionContext,
           ) -> Result<FileReadResult, ExecutionError>
```

### Invalid Method Signature

```text
error: Invalid method signature for execute_file_read
  --> src/lib.rs:10:5
   |
10 |     async fn execute_file_read(&self, op: FileReadOperation) -> Result<...> {
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: Expected signature:
           async fn execute_file_read(
               &self,
               operation: FileReadOperation,
               context: ExecutionContext,
           ) -> Result<FileReadResult, ExecutionError>
   = help: Parameter names must be exactly 'operation' and 'context'
```

### Invalid Operations Attribute

```text
error: Invalid operations attribute
  --> src/lib.rs:3:11
   |
3  | #[executor(operations = [InvalidDomain])]
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: Valid domains: Filesystem, Process, Network
   = help: Example: #[executor(operations = [Filesystem, Process])]
```

## Generated Code

The `#[executor]` macro generates:

1. **OSExecutor trait implementations** for each specified operation
2. **Type-safe delegations** to user-defined methods
3. **Error context enrichment** with operation metadata
4. **Debug implementations** for better diagnostics

### Example Generated Code

For this input:

```rust
#[executor(operations = [Filesystem])]
struct MyExecutor;

impl MyExecutor {
    async fn execute_file_read(
        &self,
        operation: FileReadOperation,
        context: ExecutionContext,
    ) -> Result<FileReadResult, ExecutionError> {
        Ok(FileReadResult {
            content: vec![],
            bytes_read: 0,
            completed_at: Utc::now(),
        })
    }
}
```

The macro generates (simplified):

```rust
#[automatically_derived]
impl OSExecutor<FileReadOperation> for MyExecutor {
    async fn execute(
        &self,
        operation: FileReadOperation,
        context: ExecutionContext,
    ) -> Result<FileReadResult, ExecutionError> {
        self.execute_file_read(operation, context).await
    }
}

// Similar implementations for FileWrite, FileDelete, DirectoryCreate, DirectoryList
```

## Best Practices

### 1. Start with Single Domain

Begin with one operation domain and expand as needed:

```rust
#[executor(operations = [Filesystem])]
struct MyExecutor;
```

### 2. Use Custom Names for Debugging

Provide descriptive names for logging:

```rust
#[executor(name = "EncryptedFileSystemExecutor")]
struct MyExecutor;
```

### 3. Preserve State with Structs

Use struct fields for stateful executors:

```rust
#[executor(operations = [Filesystem])]
struct StatefulExecutor {
    config: Config,
    cache: Cache,
}
```

### 4. Implement Helper Methods

Extract common logic into helper methods:

```rust
impl MyExecutor {
    fn validate_path(&self, path: &str) -> Result<(), ExecutionError> {
        // Validation logic
    }

    async fn execute_file_read(&self, ...) -> Result<...> {
        self.validate_path(&operation.path)?;
        // Implementation
    }
}
```

### 5. Use Type Annotations for Multi-Domain

When implementing multiple domains, use explicit type annotations to avoid ambiguity:

```rust
let result = <MyExecutor as OSExecutor<FileReadOperation>>::execute(
    &executor,
    operation,
    context
).await?;
```

## See Also

- [Custom Executor Guide](../guides/custom-executors.md) - Comprehensive guide with examples
- [Core Types](core-types.md) - Operation and result types reference
- [Examples](../../examples/custom_executor_with_macro.rs) - Full working examples

## Crate Documentation

For detailed crate-level documentation, see:
- [airssys-osl-macros on docs.rs](https://docs.rs/airssys-osl-macros)
- Source code: `airssys-osl-macros/src/lib.rs`
