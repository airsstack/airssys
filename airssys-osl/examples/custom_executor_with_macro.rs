//! Demonstrates using the #[executor] macro for custom executors
//!
//! This example shows how the #[executor] macro dramatically reduces
//! boilerplate when creating custom executors for airssys-osl.
//!
//! ## The Problem: Manual Implementation
//!
//! Without the macro, implementing a custom executor requires:
//! - Manual trait implementation for OSExecutor<O>
//! - Boilerplate for name() and supported_operation_types()
//! - Manual execute() method delegation
//! - Separate impl block for each operation type
//!
//! This can result in 100+ lines of repetitive code.
//!
//! ## The Solution: #[executor] Macro
//!
//! The macro automatically:
//! - Detects operation methods by signature
//! - Generates all required trait implementations
//! - Creates proper execute() delegation
//! - Supports custom configuration via attributes
//!
//! Result: ~85% code reduction with compile-time safety.

use airssys_osl::prelude::*;

// =============================================================================
// Example 1: Simple Single-Operation Executor
// =============================================================================

/// A custom executor that only handles file read operations.
#[derive(Debug)]
struct SimpleFileExecutor;

#[executor]
impl SimpleFileExecutor {
    async fn file_read(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        let _ = context;
        println!("Reading file: {}", operation.path);

        // Simulate reading file content
        let content = format!("Content of {}", operation.path);

        Ok(ExecutionResult::success(content.into_bytes()))
    }
}

// =============================================================================
// Example 2: Multi-Operation Executor
// =============================================================================

/// A custom executor that handles multiple filesystem operations.
#[derive(Debug)]
struct FilesystemExecutor {
    cache_enabled: bool,
}

impl FilesystemExecutor {
    fn new(cache_enabled: bool) -> Self {
        Self { cache_enabled }
    }
}

#[executor]
impl FilesystemExecutor {
    async fn file_read(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        println!(
            "Reading file: {} (cache: {})",
            operation.path, self.cache_enabled
        );

        let _ = context;
        let content = format!("File content from {}", operation.path);
        Ok(ExecutionResult::success(content.into_bytes()))
    }

    async fn file_write(
        &self,
        operation: FileWriteOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        let _ = context;
        println!(
            "Writing {} bytes to: {}",
            operation.content.len(),
            operation.path
        );

        Ok(ExecutionResult::success(b"Write successful".to_vec()))
    }

    async fn file_delete(
        &self,
        operation: FileDeleteOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        println!("Deleting file: {}", operation.path);

        let _ = context;
        Ok(ExecutionResult::success(b"Delete successful".to_vec()))
    }
}

// =============================================================================
// Example 3: Custom Configuration
// =============================================================================

/// An executor with custom name and specific operation types.
#[derive(Debug)]
struct MyCustomExecutor;

#[executor(name = "AdvancedFileSystem", operations = [Filesystem])]
impl MyCustomExecutor {
    async fn file_read(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        let _ = (operation, context);
        println!("Custom executor: file_read");
        Ok(ExecutionResult::success(b"custom result".to_vec()))
    }
}

// =============================================================================
// Example 4: Cross-Domain Executor
// =============================================================================

/// An executor that handles operations across multiple domains.
#[derive(Debug)]
struct MultiDomainExecutor;

#[executor]
impl MultiDomainExecutor {
    // Filesystem operation
    async fn file_read(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        let _ = (operation, context);
        println!("Multi-domain: file_read");
        Ok(ExecutionResult::success(b"file data".to_vec()))
    }

    // Process operation
    async fn process_spawn(
        &self,
        operation: ProcessSpawnOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        println!("Multi-domain: spawning process '{}'", operation.command);
        let _ = context;
        Ok(ExecutionResult::success(b"process spawned".to_vec()))
    }

    // Network operation
    async fn network_connect(
        &self,
        operation: NetworkConnectOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        println!("Multi-domain: connecting to {}", operation.address);
        let _ = context;
        Ok(ExecutionResult::success(b"connected".to_vec()))
    }
}

// =============================================================================
// Example 5: Executor with Helper Methods
// =============================================================================

/// An executor that includes helper methods alongside operation methods.
#[derive(Debug)]
struct CachedFileExecutor {
    cache: std::collections::HashMap<String, Vec<u8>>,
}

impl CachedFileExecutor {
    fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
        }
    }
}

#[executor]
impl CachedFileExecutor {
    // Operation method - detected by the macro
    async fn file_read(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        let _ = context;

        // Use helper method
        if let Some(cached) = self.get_from_cache(&operation.path) {
            println!("Cache hit for: {}", operation.path);
            return Ok(ExecutionResult::success(cached.clone()));
        }

        println!("Cache miss for: {}", operation.path);
        let content = format!("Fresh content from {}", operation.path);
        Ok(ExecutionResult::success(content.into_bytes()))
    }

    // Helper method - ignored by the macro
    fn get_from_cache(&self, path: &str) -> Option<&Vec<u8>> {
        self.cache.get(path)
    }

    // Another helper method
    fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

// =============================================================================
// Main: Demonstration
// =============================================================================

#[tokio::main]
async fn main() -> OSResult<()> {
    println!("=== airssys-osl #[executor] Macro Examples ===\n");

    // Example 1: Simple executor
    println!("--- Example 1: Simple Single-Operation Executor ---");
    let simple = SimpleFileExecutor;
    let context = ExecutionContext::new(SecurityContext::new("demo-user".to_string()));

    let operation = FileReadOperation::new("/tmp/test.txt");
    let result = simple.execute(operation, &context).await?;
    println!("Result: {} bytes\n", result.output.len());

    // Example 2: Multi-operation executor
    println!("--- Example 2: Multi-Operation Executor ---");
    let fs_executor = FilesystemExecutor::new(true);

    let read_op = FileReadOperation::new("/tmp/data.txt");
    fs_executor.execute(read_op, &context).await?;

    let write_op = FileWriteOperation::new("/tmp/output.txt", b"Hello, World!".to_vec());
    fs_executor.execute(write_op, &context).await?;

    let delete_op = FileDeleteOperation::new("/tmp/old.txt");
    fs_executor.execute(delete_op, &context).await?;
    println!();

    // Example 3: Custom configuration
    println!("--- Example 3: Custom Configuration ---");
    let custom = MyCustomExecutor;
    use airssys_osl::core::executor::OSExecutor;
    println!("Executor name: {}", custom.name());
    println!("Supported types: {:?}", custom.supported_operation_types());
    println!();

    // Example 4: Cross-domain executor
    println!("--- Example 4: Cross-Domain Executor ---");
    let multi = MultiDomainExecutor;

    multi
        .execute(FileReadOperation::new("/tmp/file.txt"), &context)
        .await?;
    multi
        .execute(ProcessSpawnOperation::new("echo".to_string()), &context)
        .await?;
    multi
        .execute(
            NetworkConnectOperation::new("127.0.0.1:8080".to_string()),
            &context,
        )
        .await?;
    println!();

    // Example 5: Executor with helpers
    println!("--- Example 5: Executor with Helper Methods ---");
    let cached = CachedFileExecutor::new();
    println!("Cache size: {}", cached.cache_size());

    cached
        .execute(FileReadOperation::new("/tmp/cached.txt"), &context)
        .await?;
    println!();

    println!("=== All examples completed successfully! ===");
    Ok(())
}
