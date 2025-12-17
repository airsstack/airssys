# Custom Executors

This guide explains how to create custom executors for airssys-osl using the `#[executor]` macro.

## Table of Contents
- [The Problem: Boilerplate Code](#the-problem-boilerplate-code)
- [The Solution: #[executor] Macro](#the-solution-executor-macro)
- [Basic Usage](#basic-usage)
- [Method Signature Requirements](#method-signature-requirements)
- [Custom Configuration](#custom-configuration)
- [Supported Operations](#supported-operations)
- [Advanced Features](#advanced-features)

## The Problem: Boilerplate Code

Without the `#[executor]` macro, implementing a custom executor requires significant boilerplate:

```rust
use airssys_osl::prelude::*;
use async_trait::async_trait;

struct MyExecutor;

// Manual trait implementation for EACH operation type
#[async_trait]
impl OSExecutor<FileReadOperation> for MyExecutor {
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
        // Your implementation
        todo!()
    }
}

// Repeat for FileWriteOperation, FileDeleteOperation, etc.
// Each operation requires ~20 lines of boilerplate!
```

This can result in **100+ lines of repetitive code** for a multi-operation executor.

## The Solution: #[executor] Macro

The `#[executor]` macro **automatically generates all trait implementations** by detecting operation methods based on their signatures:

```rust
use airssys_osl::prelude::*;

#[derive(Debug)]
struct MyExecutor;

#[executor]
impl MyExecutor {
    async fn file_read(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        // Your implementation
        todo!()
    }
    
    async fn file_write(
        &self,
        operation: FileWriteOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        // Your implementation
        todo!()
    }
}
```

**Result:** ~85% code reduction with full compile-time safety!

## Basic Usage

### Single Operation Executor

The simplest use case - an executor that handles one type of operation:

```rust
use airssys_osl::prelude::*;

#[derive(Debug)]
struct SimpleFileReader;

#[executor]
impl SimpleFileReader {
    async fn file_read(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        // Read the file
        let content = std::fs::read(&operation.path)
            .map_err(|e| OSError::io_error(format!("Failed to read file: {}", e)))?;
        
        Ok(ExecutionResult::success(content))
    }
}
```

### Multiple Operations Executor

Handle multiple operations in the same executor:

```rust
#[derive(Debug)]
struct FilesystemExecutor;

#[executor]
impl FilesystemExecutor {
    async fn file_read(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        // Implementation
        todo!()
    }

    async fn file_write(
        &self,
        operation: FileWriteOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        // Implementation
        todo!()
    }

    async fn file_delete(
        &self,
        operation: FileDeleteOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        // Implementation
        todo!()
    }
}
```

The macro generates **3 separate trait implementations** automatically!

## Method Signature Requirements

For the macro to detect operation methods, they must follow strict signature rules:

### Required Elements

1. **`async` keyword**: Method must be async
   ```rust
   async fn file_read(...)  // ✅ Correct
   fn file_read(...)        // ❌ Error: must be async
   ```

2. **`&self` receiver**: Must use shared reference
   ```rust
   async fn file_read(&self, ...)      // ✅ Correct
   async fn file_read(&mut self, ...)  // ❌ Error: cannot mutate
   async fn file_read(self, ...)       // ❌ Error: must be reference
   ```

3. **Exactly 2 parameters** with **exact names**:
   ```rust
   async fn file_read(
       &self,
       operation: FileReadOperation,  // ✅ Must be named 'operation'
       context: &ExecutionContext,    // ✅ Must be named 'context'
   ) -> OSResult<ExecutionResult>
   ```

4. **Return type**: Must be `OSResult<ExecutionResult>`
   ```rust
   -> OSResult<ExecutionResult>  // ✅ Correct
   -> Result<...>                // ❌ Error: wrong return type
   ```

### Operation Method Names

The macro recognizes these operation method names:

**Filesystem Operations (5):**

- `file_read` → `FileReadOperation`
- `file_write` → `FileWriteOperation`
- `file_delete` → `FileDeleteOperation`
- `directory_create` → `DirectoryCreateOperation`
- `directory_list` → `DirectoryListOperation`

**Process Operations (3):**

- `process_spawn` → `ProcessSpawnOperation`
- `process_kill` → `ProcessKillOperation`
- `process_signal` → `ProcessSignalOperation`

**Network Operations (3):**

- `network_connect` → `NetworkConnectOperation`
- `network_listen` → `NetworkListenOperation`
- `network_socket` → `NetworkSocketOperation`

## Custom Configuration

The macro supports custom configuration via attributes:

### Custom Executor Name

Override the auto-detected executor name:

```rust
#[derive(Debug)]
struct MyExecutor;

#[executor(name = "AdvancedFileSystem")]
impl MyExecutor {
    async fn file_read(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        todo!()
    }
}

// executor.name() returns "AdvancedFileSystem" instead of "MyExecutor"
```

### Custom Operation Types

Explicitly specify which operation types the executor supports:

```rust
#[executor(operations = [Filesystem, Process])]
impl MyExecutor {
    async fn file_read(...) { }
    async fn process_spawn(...) { }
}
```

Valid operation types: `Filesystem`, `Process`, `Network`

### Combined Configuration

Use both `name` and `operations` together:

```rust
#[executor(name = "CustomExecutor", operations = [Filesystem])]
impl MyExecutor {
    async fn file_read(...) { }
}
```

## Supported Operations

The macro currently supports **11 operations** across 3 domains:

| Domain | Operations | Count |
|--------|-----------|-------|
| **Filesystem** | file_read, file_write, file_delete, directory_create, directory_list | 5 |
| **Process** | process_spawn, process_kill, process_signal | 3 |
| **Network** | network_connect, network_listen, network_socket | 3 |
| **Total** | | **11** |

### Cross-Domain Executors

You can mix operations from different domains in a single executor:

```rust
#[derive(Debug)]
struct MultiDomainExecutor;

#[executor]
impl MultiDomainExecutor {
    // Filesystem
    async fn file_read(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        todo!()
    }

    // Process
    async fn process_spawn(
        &self,
        operation: ProcessSpawnOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        todo!()
    }

    // Network
    async fn network_connect(
        &self,
        operation: NetworkConnectOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        todo!()
    }
}
```

The macro generates trait implementations for all 3 operation types!

## Advanced Features

### Helper Methods

You can add helper methods alongside operation methods - the macro ignores non-operation methods:

```rust
#[derive(Debug)]
struct CachedExecutor {
    cache: HashMap<String, Vec<u8>>,
}

#[executor]
impl CachedExecutor {
    // Operation method - detected by macro
    async fn file_read(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        // Use helper method
        if let Some(cached) = self.get_from_cache(&operation.path) {
            return Ok(ExecutionResult::success(cached.clone()));
        }
        
        // Read from disk
        todo!()
    }

    // Helper method - ignored by macro
    fn get_from_cache(&self, path: &str) -> Option<&Vec<u8>> {
        self.cache.get(path)
    }
    
    // Another helper
    fn cache_size(&self) -> usize {
        self.cache.len()
    }
}
```

**Only methods matching the operation signature pattern are treated as operations.**

### Stateful Executors

Executors can have state:

```rust
#[derive(Debug)]
struct StatefulExecutor {
    config: ExecutorConfig,
    metrics: Arc<Mutex<Metrics>>,
}

impl StatefulExecutor {
    fn new(config: ExecutorConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(Mutex::new(Metrics::default())),
        }
    }
}

#[executor]
impl StatefulExecutor {
    async fn file_read(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        // Access state
        if !self.config.allow_reads {
            return Err(OSError::permission_denied("Reads disabled"));
        }
        
        // Update metrics
        self.metrics.lock().unwrap().reads += 1;
        
        // Perform operation
        todo!()
    }
}
```

### Error Messages

The macro provides helpful compile-time error messages:

```rust
#[executor]
impl MyExecutor {
    // ❌ Error: Method must be async
    fn file_read(&self, operation: FileReadOperation, context: &ExecutionContext) 
        -> OSResult<ExecutionResult> { }
    
    // ❌ Error: Second parameter must be named 'context', found 'ctx'
    async fn file_write(&self, operation: FileWriteOperation, ctx: &ExecutionContext) 
        -> OSResult<ExecutionResult> { }
    
    // ❌ Error: Expected exactly 2 parameters, found 1
    async fn file_delete(&self, operation: FileDeleteOperation) 
        -> OSResult<ExecutionResult> { }
}
```

### Feature Flag

The macro is enabled by default via the `macros` feature:

```toml
[dependencies]
airssys-osl = "0.1"  # Macros enabled

# Or explicitly disable:
airssys-osl = { version = "0.1", default-features = false }
```

## Complete Example

See [`examples/custom_executor_with_macro.rs`](https://github.com/airsstack/airssys/blob/main/airssys-osl/examples/custom_executor_with_macro.rs) for a comprehensive demonstration covering:

- Simple single-operation executors
- Multi-operation executors
- Custom configuration
- Cross-domain executors
- Executors with helper methods
- Stateful executors

## Next Steps

- Review [API Reference](../api/macros.md) for detailed macro documentation
- Explore [Examples](https://github.com/airsstack/airssys/tree/main/airssys-osl/examples)
- Learn about [Middleware](./middleware.md) integration
- Understand [Security Context](./security-setup.md) usage
