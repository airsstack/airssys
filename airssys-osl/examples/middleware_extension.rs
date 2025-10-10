//! Middleware extension trait example for airssys-osl
//!
//! This example demonstrates how to use the ExecutorExt trait to add middleware
//! capabilities to any executor with a fluent API.
//! Run with: cargo run --example middleware_extension

#![allow(clippy::unwrap_used, clippy::expect_used)] // Allow in examples for clarity

use airssys_osl::core::executor::OSExecutor; // Import the executor trait
use airssys_osl::executors::{FilesystemExecutor, ProcessExecutor};
use airssys_osl::middleware::logger::{
    ConsoleActivityLogger, FileActivityLogger, LogFormat, LoggerMiddleware, TracingActivityLogger,
};
use airssys_osl::middleware::ExecutorExt; // Import the extension trait
use airssys_osl::prelude::*;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> Result<(), OSError> {
    println!("=== AirsSys OSL Middleware Extension Example ===\n");

    // Initialize tracing for logger output
    tracing_subscriber::fmt::init();

    // Create temporary directory for examples
    let temp_dir = TempDir::new().map_err(|e| {
        OSError::execution_failed(format!("Failed to create temp directory: {e}"))
    })?;
    let temp_path = temp_dir.path();

    // ==========================================
    // Example 1: Basic Middleware Extension
    // ==========================================
    println!("📝 Example 1: Basic Middleware Extension");
    println!("=========================================\n");

    // Create a logger middleware
    let logger = ConsoleActivityLogger::default();
    let middleware = LoggerMiddleware::with_default_config(logger);

    // Create an executor and add middleware using the extension trait
    let executor = FilesystemExecutor::default().with_middleware(middleware);

    // Execute an operation - middleware will automatically log the activity
    let file_path = temp_path.join("example1.txt");
    let operation = FileWriteOperation::new(
        file_path.to_string_lossy().to_string(),
        b"Hello from middleware!".to_vec(),
    );
    let security_context = SecurityContext::new("example-user".to_string());
    let context = ExecutionContext::new(security_context);

    println!("Executing file write with logging middleware...");
    let _result = executor.execute(operation, &context).await?;
    println!("✓ Operation completed successfully\n");

    // ==========================================
    // Example 2: Chaining Multiple Middleware
    // ==========================================
    println!("🔗 Example 2: Chaining Multiple Middleware");
    println!("===========================================\n");

    // Create first middleware (console logger)
    let console_logger = ConsoleActivityLogger::new().with_format(LogFormat::Compact);
    let console_middleware = LoggerMiddleware::with_default_config(console_logger);

    // Create second middleware (file logger)
    let log_file_path = temp_path.join("operations.log");
    let file_logger = FileActivityLogger::new(log_file_path.clone())
        .await
        .expect("Failed to create file logger");
    let file_middleware = LoggerMiddleware::with_default_config(file_logger);

    // Chain middleware using the extension trait
    let executor = FilesystemExecutor::default()
        .with_middleware(console_middleware)
        .with_middleware(file_middleware);

    // Execute an operation - both middleware will process it
    let file_path = temp_path.join("example2.txt");
    let operation = FileWriteOperation::new(
        file_path.to_string_lossy().to_string(),
        b"Testing middleware chain!".to_vec(),
    );
    let security_context = SecurityContext::new("example-user".to_string());
    let context = ExecutionContext::new(security_context);

    println!("Executing file write with chained middleware...");
    let _result = executor.execute(operation, &context).await?;
    println!("✓ Operation completed successfully");
    println!(
        "✓ Logs written to both console and file: {log_file_path:?}\n"
    );

    // ==========================================
    // Example 3: Middleware with Different Operations
    // ==========================================
    println!("⚙️ Example 3: Middleware with Different Operations");
    println!("===================================================\n");

    // Create tracing middleware (note: can't clone middleware, so create two instances)
    let tracing_logger1 = TracingActivityLogger::new();
    let tracing_middleware1 = LoggerMiddleware::with_default_config(tracing_logger1);

    let tracing_logger2 = TracingActivityLogger::new();
    let tracing_middleware2 = LoggerMiddleware::with_default_config(tracing_logger2);

    // Use middleware with filesystem operations
    println!("1. Filesystem operation with middleware:");
    let fs_executor = FilesystemExecutor::default().with_middleware(tracing_middleware1);
    let read_op =
        FileReadOperation::new(temp_path.join("example1.txt").to_string_lossy().to_string());
    let security_context = SecurityContext::new("example-user".to_string());
    let context = ExecutionContext::new(security_context);
    let result = fs_executor.execute(read_op, &context).await?;
    println!("   ✓ Read {} bytes\n", result.output.len());

    // Use middleware with process operations
    println!("2. Process operation with middleware:");
    let process_executor =
        ProcessExecutor::new("example-process-executor").with_middleware(tracing_middleware2);
    let spawn_op = ProcessSpawnOperation::new("echo".to_string())
        .with_args(vec!["Middleware works!".to_string()]);
    let security_context = SecurityContext::new("example-user".to_string());
    let context = ExecutionContext::new(security_context);
    let _result = process_executor.execute(spawn_op, &context).await?;
    println!("   ✓ Process spawned successfully\n");

    // ==========================================
    // Summary
    // ==========================================
    println!("=== Summary ===");
    println!("✅ Demonstrated basic middleware extension with single middleware");
    println!("✅ Demonstrated chaining multiple middleware instances");
    println!("✅ Demonstrated middleware working across different executor types");
    println!("\n🎉 All middleware extension examples completed successfully!");

    Ok(())
}
