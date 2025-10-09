# Middleware Guide

This guide explains how to use and compose middleware in AirsSys OSL to add cross-cutting concerns like logging, security, metrics, and custom processing to your executors.

## Overview

Middleware in AirsSys OSL provides a flexible way to add functionality to executors without modifying their core logic. Middleware can:

- Log operations and results
- Validate security policies
- Collect metrics and telemetry
- Transform operations before execution
- Handle errors with custom logic
- Add retry, timeout, or rate-limiting capabilities

## Core Concepts

### Middleware Trait

All middleware must implement the `Middleware<O>` trait where `O` is the operation type:

```rust
use airssys_osl::core::middleware::{Middleware, MiddlewareResult, ErrorAction};

#[async_trait]
pub trait Middleware<O: Operation>: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    fn priority(&self) -> u32;
    
    async fn can_process(&self, operation: &O, context: &ExecutionContext) -> bool;
    
    async fn before_execution(
        &self,
        operation: O,
        context: &ExecutionContext,
    ) -> MiddlewareResult<Option<O>>;
    
    async fn after_execution(
        &self,
        context: &ExecutionContext,
        result: &OSResult<ExecutionResult>,
    ) -> MiddlewareResult<()>;
    
    async fn handle_error(
        &self,
        error: OSError,
        context: &ExecutionContext,
    ) -> ErrorAction;
}
```

### Middleware Lifecycle

When an operation is executed through a middleware-wrapped executor:

1. **can_process** - Check if middleware applies to this operation
2. **before_execution** - Transform or validate operation before execution
3. **execute** - Core executor runs the operation
4. **after_execution** - Post-process results and log outcomes
5. **handle_error** - Handle any errors that occurred (if applicable)

## Basic Usage

### Using Built-in Middleware

AirsSys OSL provides several built-in middleware implementations:

#### Logger Middleware

The `LoggerMiddleware` logs all operations and their results:

```rust
use airssys_osl::prelude::*;
use airssys_osl::middleware::logger::{ConsoleActivityLogger, LoggerMiddleware};
use airssys_osl::middleware::ExecutorExt;

#[tokio::main]
async fn main() -> Result<(), OSError> {
    // Create a logger
    let logger = ConsoleActivityLogger::default();
    let middleware = LoggerMiddleware::with_default_config(logger);
    
    // Wrap executor with middleware
    let executor = FilesystemExecutor::default()
        .with_middleware(middleware);
    
    // Execute operation - automatically logged
    let operation = FileReadOperation::new("/tmp/test.txt".to_string());
    let context = ExecutionContext::new(SecurityContext::new("user".to_string()));
    let result = executor.execute(operation, &context).await?;
    
    Ok(())
}
```

#### Available Loggers

- **ConsoleActivityLogger** - Logs to stdout with configurable format
- **FileActivityLogger** - Logs to a file with rotation support
- **TracingActivityLogger** - Integrates with the `tracing` ecosystem

## Middleware Chaining

One of the most powerful features is the ability to chain multiple middleware together. This creates a processing pipeline where each middleware can add its own functionality.

### How Chaining Works

The `ExecutorExt` trait provides the `.with_middleware()` method that can be called multiple times:

```rust
use airssys_osl::prelude::*;
use airssys_osl::middleware::ExecutorExt;

let executor = FilesystemExecutor::default()
    .with_middleware(middleware1)
    .with_middleware(middleware2)
    .with_middleware(middleware3);
```

### Execution Order

When multiple middleware are chained, they execute in a **nested/onion-like** pattern:

```
Request Flow:
  → middleware3.before_execution()
    → middleware2.before_execution()
      → middleware1.before_execution()
        → [Core Executor]
      ← middleware1.after_execution()
    ← middleware2.after_execution()
  ← middleware3.after_execution()
```

**Key Points:**
- **Outermost middleware runs first** for `before_execution` hooks
- **Innermost middleware is closest** to the actual executor
- **Reverse order** for `after_execution` and error handling
- Each middleware can transform the operation or short-circuit the chain

### Chaining Example

```rust
use airssys_osl::prelude::*;
use airssys_osl::middleware::logger::{
    ConsoleActivityLogger, FileActivityLogger, LoggerMiddleware
};
use airssys_osl::middleware::ExecutorExt;
use airssys_osl::core::executor::OSExecutor;

#[tokio::main]
async fn main() -> Result<(), OSError> {
    // Create multiple middleware
    let console_logger = ConsoleActivityLogger::default();
    let console_middleware = LoggerMiddleware::with_default_config(console_logger);
    
    let file_logger = FileActivityLogger::new("/tmp/ops.log").await?;
    let file_middleware = LoggerMiddleware::with_default_config(file_logger);
    
    // Chain them together
    let executor = FilesystemExecutor::default()
        .with_middleware(console_middleware)  // Logs to console
        .with_middleware(file_middleware);     // Also logs to file
    
    // Execute operation - both middleware will process it
    let operation = FileWriteOperation::new(
        "/tmp/test.txt".to_string(),
        b"Hello, Middleware!".to_vec()
    );
    let context = ExecutionContext::new(SecurityContext::new("user".to_string()));
    let result = executor.execute(operation, &context).await?;
    
    // Both console and file will have logged this operation
    Ok(())
}
```

## Middleware Across Executor Types

The same middleware can be used with different executor types:

```rust
use airssys_osl::prelude::*;
use airssys_osl::executors::{FilesystemExecutor, ProcessExecutor};
use airssys_osl::middleware::logger::{TracingActivityLogger, LoggerMiddleware};
use airssys_osl::middleware::ExecutorExt;

#[tokio::main]
async fn main() -> Result<(), OSError> {
    let logger = TracingActivityLogger::new();
    
    // Same middleware, different executors
    let fs_executor = FilesystemExecutor::default()
        .with_middleware(LoggerMiddleware::with_default_config(logger.clone()));
    
    let process_executor = ProcessExecutor::new("my-executor")
        .with_middleware(LoggerMiddleware::with_default_config(logger));
    
    // Both executors now have logging capabilities
    Ok(())
}
```

## Creating Custom Middleware

You can create custom middleware to implement your own cross-cutting concerns:

```rust
use airssys_osl::prelude::*;
use airssys_osl::core::middleware::{Middleware, MiddlewareResult, ErrorAction};
use async_trait::async_trait;

#[derive(Debug)]
struct MetricsMiddleware {
    // Your metrics collector
}

#[async_trait]
impl<O> Middleware<O> for MetricsMiddleware
where
    O: Operation + Send + Sync + std::fmt::Debug,
{
    fn name(&self) -> &str {
        "metrics_middleware"
    }
    
    fn priority(&self) -> u32 {
        100
    }
    
    async fn can_process(&self, _operation: &O, _context: &ExecutionContext) -> bool {
        true  // Process all operations
    }
    
    async fn before_execution(
        &self,
        operation: O,
        _context: &ExecutionContext,
    ) -> MiddlewareResult<Option<O>> {
        // Record operation start time
        Ok(Some(operation))
    }
    
    async fn after_execution(
        &self,
        _context: &ExecutionContext,
        result: &OSResult<ExecutionResult>,
    ) -> MiddlewareResult<()> {
        // Record operation completion and metrics
        if result.is_ok() {
            // Record success metric
        } else {
            // Record failure metric
        }
        Ok(())
    }
    
    async fn handle_error(
        &self,
        _error: OSError,
        _context: &ExecutionContext,
    ) -> ErrorAction {
        ErrorAction::Continue
    }
}
```

## Best Practices

### Middleware Ordering

Order your middleware chain carefully based on concerns:

```rust
// Recommended order:
let executor = FilesystemExecutor::default()
    .with_middleware(metrics_middleware)      // Outermost - measure everything
    .with_middleware(security_middleware)     // Security validation
    .with_middleware(retry_middleware)        // Retry failed operations
    .with_middleware(logging_middleware);     // Innermost - detailed logging
```

### Error Handling

Middleware can decide how to handle errors:

- **ErrorAction::Stop** - Stop processing and return the error
- **ErrorAction::Continue** - Continue with the error
- **ErrorAction::Retry** - Attempt to retry the operation

### Performance Considerations

- Keep `can_process()` lightweight - it's called for every operation
- Avoid blocking operations in middleware hooks
- Use async operations for I/O (logging to files, network calls)
- Consider middleware overhead when chaining many middleware

### Testing Middleware

Test your middleware in isolation:

```rust
#[tokio::test]
async fn test_custom_middleware() {
    let middleware = MyCustomMiddleware::new();
    let operation = FileReadOperation::new("/test".to_string());
    let context = ExecutionContext::new(SecurityContext::new("test".to_string()));
    
    // Test can_process
    assert!(middleware.can_process(&operation, &context).await);
    
    // Test before_execution
    let result = middleware.before_execution(operation, &context).await;
    assert!(result.is_ok());
}
```

## Advanced Patterns

### Conditional Middleware

Middleware can selectively process operations:

```rust
async fn can_process(&self, operation: &O, context: &ExecutionContext) -> bool {
    // Only process operations from specific users
    context.security_context.user() == "admin"
}
```

### Operation Transformation

Middleware can modify operations before execution:

```rust
async fn before_execution(
    &self,
    mut operation: O,
    _context: &ExecutionContext,
) -> MiddlewareResult<Option<O>> {
    // Modify operation parameters
    // For example, sanitize file paths
    Ok(Some(operation))
}
```

### Short-Circuit Execution

Middleware can handle operations without calling the executor:

```rust
async fn before_execution(
    &self,
    operation: O,
    context: &ExecutionContext,
) -> MiddlewareResult<Option<O>> {
    if should_cache_hit(&operation) {
        // Return None to skip executor and use cached result
        return Ok(None);
    }
    Ok(Some(operation))
}
```

## Examples

See the complete working examples in the repository:

- **[middleware_extension.rs](../../../examples/middleware_extension.rs)** - Demonstrates basic usage, chaining, and cross-executor middleware
- **[helper_functions.rs](../../../examples/helper_functions.rs)** - Shows helper functions that use middleware internally

## Next Steps

- Learn about [Security Setup](./security-setup.md) for security middleware
- Explore [Logging Configuration](./logging-config.md) for advanced logging patterns
- Read about [Custom Executors](./custom-executors.md) to build executor/middleware combinations

## Related Documentation

- [Core Types API](../api/core-types.md)
- [Activity Logging API](../api/logging.md)
- [Architecture Overview](../architecture/README.md)
