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
async fn {operation_name}(
    &self,
    operation: {OperationType},
    context: &ExecutionContext,
) -> OSResult<ExecutionResult>
```

**Requirements**:
- Method must be `async`
- Method name must match the operation name (e.g., `file_read`, not `execute_file_read`)
- First parameter must be named `operation`
- Second parameter must be named `context` and type `&ExecutionContext`
- Return type must be `OSResult<ExecutionResult>`

##### Operation Method Mapping

| Operation Domain | Method Name | Operation Type |
|------------------|-------------|----------------|
| **Filesystem** | | |
| File Read | `file_read` | `FileReadOperation` |
| File Write | `file_write` | `FileWriteOperation` |
| File Delete | `file_delete` | `FileDeleteOperation` |
| Directory Create | `directory_create` | `DirectoryCreateOperation` |
| Directory List | `directory_list` | `DirectoryListOperation` |
| **Process** | | |
| Process Spawn | `process_spawn` | `ProcessSpawnOperation` |
| Process Kill | `process_kill` | `ProcessKillOperation` |
| Process Signal | `process_signal` | `ProcessSignalOperation` |
| **Network** | | |
| Network Connect | `network_connect` | `NetworkConnectOperation` |
| Network Listen | `network_listen` | `NetworkListenOperation` |
| Network Socket | `network_socket` | `NetworkSocketOperation` |

#### Examples

##### Single Domain Executor

```rust
use airssys_osl::prelude::*;

#[executor(operations = [Filesystem])]
struct FileSystemExecutor;

impl FileSystemExecutor {
    async fn file_read(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        // Custom implementation
        Ok(ExecutionResult::success(vec![]))
    }

    async fn file_write(
        &self,
        operation: FileWriteOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        // Custom implementation
        Ok(ExecutionResult::success(vec![]))
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
    async fn file_read(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        // Implementation
        Ok(ExecutionResult::success(vec![]))
    }

    // Implement required methods for Process operations
    async fn process_spawn(
        &self,
        operation: ProcessSpawnOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        // Implementation
        Ok(ExecutionResult::success(vec![]))
    }

    // Implement required methods for Network operations
    async fn network_connect(
        &self,
        operation: NetworkConnectOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        // Implementation
        Ok(ExecutionResult::success(vec![]))
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

    async fn file_read(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        // Use helper method
        if let Some(cached) = self.get_from_cache(&operation.path) {
            return Ok(ExecutionResult::success(cached.clone()));
        }
        // ... fallback logic
        Ok(ExecutionResult::success(vec![]))
    }
}
```

## Error Messages

The macro provides helpful error messages for common mistakes:

### Missing Method Implementation

```text
error: No operation methods found
  --> src/lib.rs:5:1
   |
5  | #[executor]
   | ^^^^^^^^^^^
   |
   = note: Expected methods named: file_read, file_write, file_delete, 
           directory_create, directory_list, process_spawn, process_kill, 
           process_signal, network_connect, network_listen, network_socket
```

### Invalid Method Signature

```text
error: First parameter must be named 'operation', found 'op'
  --> src/lib.rs:10:5
   |
10 |     async fn file_read(&self, op: FileReadOperation, context: &ExecutionContext) {
   |                               ^^
   |
   = help: Parameter names must be exactly 'operation' and 'context'
```

### Invalid Operations Attribute

```text
error: Unknown operation type 'InvalidDomain'
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
    async fn file_read(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        Ok(ExecutionResult::success(vec![]))
    }
}
```

The macro generates (simplified):

```rust
#[automatically_derived]
#[async_trait::async_trait]
impl airssys_osl::core::executor::OSExecutor<FileReadOperation> for MyExecutor {
    fn name(&self) -> &str {
        "MyExecutor"
    }

    fn supported_operation_types(&self) -> Vec<OperationType> {
        vec![OperationType::Filesystem]
    }

    async fn execute(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        self.file_read(operation, context).await
    }
}

// Similar implementations for other operations...
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
    fn validate_path(&self, path: &str) -> OSResult<()> {
        // Validation logic
        Ok(())
    }

    async fn file_read(&self, operation: FileReadOperation, context: &ExecutionContext) -> OSResult<ExecutionResult> {
        self.validate_path(&operation.path)?;
        // Implementation
        Ok(ExecutionResult::success(vec![]))
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
