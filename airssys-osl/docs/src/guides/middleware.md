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

Custom middleware allows you to implement your own cross-cutting concerns like rate limiting, caching, metrics collection, retry logic, and more. This section provides a comprehensive guide to creating production-ready custom middleware.

### Step-by-Step Guide

#### 1. Define Your Middleware Struct

Create a struct to hold your middleware state and configuration:

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use tokio::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct RateLimitMiddleware {
    /// Maximum operations allowed per second
    max_ops_per_second: u32,
    /// Shared state tracking operation timestamps
    state: Arc<Mutex<RateLimitState>>,
}

#[derive(Debug)]
struct RateLimitState {
    /// Track timestamps of recent operations per user
    operation_times: HashMap<String, Vec<Instant>>,
}
```

**Key Points:**
- Use `#[derive(Debug, Clone)]` for middleware that will be shared across executors
- Use `Arc<Mutex<T>>` for thread-safe shared state
- Separate state into its own struct for clarity

#### 2. Implement the Constructor

Provide a clear API for creating your middleware:

```rust
impl RateLimitMiddleware {
    /// Create a new rate limiter with specified operations per second limit.
    pub fn new(max_ops_per_second: u32) -> Self {
        Self {
            max_ops_per_second,
            state: Arc::new(Mutex::new(RateLimitState {
                operation_times: HashMap::new(),
            })),
        }
    }
}
```

#### 3. Implement the Middleware Trait

Implement the `Middleware<O>` trait for your middleware:

```rust
use airssys_osl::core::middleware::{Middleware, MiddlewareResult, MiddlewareError, ErrorAction};
use airssys_osl::core::operation::Operation;
use airssys_osl::core::context::ExecutionContext;
use airssys_osl::core::result::OSResult;
use async_trait::async_trait;

#[async_trait]
impl<O: Operation> Middleware<O> for RateLimitMiddleware {
    fn name(&self) -> &str {
        "rate_limiter"
    }
    
    fn priority(&self) -> u32 {
        // High priority (75) - run before most middleware but after security (100)
        75
    }
    
    async fn can_process(&self, _operation: &O, _context: &ExecutionContext) -> bool {
        // Process all operations
        true
    }
    
    async fn before_execution(
        &self,
        operation: O,
        context: &ExecutionContext,
    ) -> MiddlewareResult<Option<O>> {
        let user = &context.security_context.principal;
        
        // Check rate limit
        if self.check_rate_limit(user).await {
            // Under limit - allow operation
            Ok(Some(operation))
        } else {
            // Rate limit exceeded - reject operation
            Err(MiddlewareError::NonFatal(format!(
                "Rate limit exceeded for user '{}': max {} operations per second",
                user, self.max_ops_per_second
            )))
        }
    }
    
    async fn after_execution(
        &self,
        _context: &ExecutionContext,
        _result: &OSResult<airssys_osl::core::executor::ExecutionResult>,
    ) -> MiddlewareResult<()> {
        // No post-processing needed for rate limiting
        Ok(())
    }
    
    async fn handle_error(
        &self,
        _error: OSError,
        _context: &ExecutionContext,
    ) -> ErrorAction {
        // Let errors propagate
        ErrorAction::Stop
    }
}
```

**Trait Method Guide:**

- **`name()`**: Unique identifier for your middleware (used in logging and debugging)
- **`priority()`**: Determines execution order (0-100, higher = outer layer)
  - 100: Security middleware
  - 75: Rate limiting, caching
  - 50: Metrics, logging
  - 25: Retry logic
- **`can_process()`**: Filter which operations this middleware handles (return false to skip)
- **`before_execution()`**: Validate, transform, or reject operations before execution
  - Return `Ok(Some(operation))` to continue
  - Return `Ok(None)` to short-circuit (cached result, etc.)
  - Return `Err(...)` to reject the operation
- **`after_execution()`**: Process results, update metrics, log outcomes
- **`handle_error()`**: Custom error handling
  - `ErrorAction::Stop`: Propagate error immediately
  - `ErrorAction::Continue`: Log and continue
  - `ErrorAction::Retry`: Attempt to retry operation

#### 4. Implement Helper Methods

Add helper methods for your middleware logic:

```rust
impl RateLimitMiddleware {
    /// Check if the user has exceeded their rate limit.
    async fn check_rate_limit(&self, user: &str) -> bool {
        let mut state = self.state.lock().await;
        let now = Instant::now();
        let one_second_ago = now - Duration::from_secs(1);
        
        // Get or create user's operation history
        let times = state
            .operation_times
            .entry(user.to_string())
            .or_insert_with(Vec::new);
        
        // Remove operations older than 1 second (sliding window)
        times.retain(|&time| time > one_second_ago);
        
        // Check if under limit
        if times.len() < self.max_ops_per_second as usize {
            // Record this operation
            times.push(now);
            true
        } else {
            false
        }
    }
}
```

### Real-World Middleware Examples

#### Example 1: Rate Limiting (Complete Implementation)

See the complete working example in [`examples/custom_middleware.rs`](../../../examples/custom_middleware.rs) which demonstrates:

- Thread-safe state management with `Arc<Mutex<HashMap>>`
- Per-user rate tracking with sliding window
- Configurable operations per second limit
- Integration with ExecutorExt and helper functions

**Usage:**

```rust
use airssys_osl::middleware::ext::ExecutorExt;

// Create rate limiter: 100 operations per second
let rate_limiter = RateLimitMiddleware::new(100);

// Use with executor
let executor = FilesystemExecutor::default()
    .with_middleware(rate_limiter);

// Or use with helper functions
let data = read_file_with_middleware(
    "/path/to/file",
    "user",
    RateLimitMiddleware::new(50)
).await?;
```

#### Example 2: Caching Middleware (Conceptual)

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use tokio::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CachingMiddleware {
    cache: Arc<RwLock<HashMap<String, CachedResult>>>,
    ttl: Duration,
}

#[derive(Debug, Clone)]
struct CachedResult {
    data: Vec<u8>,
    cached_at: Instant,
}

impl CachingMiddleware {
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }
    
    async fn get_cached(&self, key: &str) -> Option<Vec<u8>> {
        let cache = self.cache.read().await;
        if let Some(entry) = cache.get(key) {
            if entry.cached_at.elapsed() < self.ttl {
                return Some(entry.data.clone());
            }
        }
        None
    }
    
    async fn set_cached(&self, key: String, data: Vec<u8>) {
        let mut cache = self.cache.write().await;
        cache.insert(key, CachedResult {
            data,
            cached_at: Instant::now(),
        });
    }
}

#[async_trait]
impl<O: Operation> Middleware<O> for CachingMiddleware {
    fn name(&self) -> &str {
        "caching"
    }
    
    fn priority(&self) -> u32 {
        75 // High priority to check cache early
    }
    
    async fn can_process(&self, operation: &O, _context: &ExecutionContext) -> bool {
        // Only cache read operations
        operation.operation_type() == OperationType::Filesystem
    }
    
    async fn before_execution(
        &self,
        operation: O,
        _context: &ExecutionContext,
    ) -> MiddlewareResult<Option<O>> {
        // Check cache - if hit, return None to skip execution
        let cache_key = format!("{:?}", operation);
        
        if self.get_cached(&cache_key).await.is_some() {
            // Cache hit - skip execution (would need to return cached result)
            // Note: This is simplified - real implementation needs result injection
            Ok(None)
        } else {
            // Cache miss - continue to executor
            Ok(Some(operation))
        }
    }
    
    async fn after_execution(
        &self,
        _context: &ExecutionContext,
        result: &OSResult<ExecutionResult>,
    ) -> MiddlewareResult<()> {
        // Cache successful results
        if let Ok(exec_result) = result {
            let cache_key = format!("{}", exec_result.operation_id);
            self.set_cached(cache_key, exec_result.output.clone()).await;
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

**Usage:**

```rust
// Cache file reads for 60 seconds
let caching = CachingMiddleware::new(Duration::from_secs(60));

let executor = FilesystemExecutor::default()
    .with_middleware(caching);
```

#### Example 3: Metrics Collection Middleware (Conceptual)

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MetricsMiddleware {
    metrics: Arc<Mutex<OperationMetrics>>,
}

#[derive(Debug, Default)]
struct OperationMetrics {
    total_ops: u64,
    successful_ops: u64,
    failed_ops: u64,
    total_duration_ms: u64,
}

impl MetricsMiddleware {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(OperationMetrics::default())),
        }
    }
    
    pub async fn get_stats(&self) -> OperationMetrics {
        self.metrics.lock().await.clone()
    }
}

#[async_trait]
impl<O: Operation> Middleware<O> for MetricsMiddleware {
    fn name(&self) -> &str {
        "metrics"
    }
    
    fn priority(&self) -> u32 {
        50 // Medium priority
    }
    
    async fn can_process(&self, _operation: &O, _context: &ExecutionContext) -> bool {
        true // Collect metrics for all operations
    }
    
    async fn before_execution(
        &self,
        operation: O,
        _context: &ExecutionContext,
    ) -> MiddlewareResult<Option<O>> {
        let mut metrics = self.metrics.lock().await;
        metrics.total_ops += 1;
        Ok(Some(operation))
    }
    
    async fn after_execution(
        &self,
        _context: &ExecutionContext,
        result: &OSResult<ExecutionResult>,
    ) -> MiddlewareResult<()> {
        let mut metrics = self.metrics.lock().await;
        
        match result {
            Ok(exec_result) => {
                metrics.successful_ops += 1;
                metrics.total_duration_ms += exec_result.duration.as_millis() as u64;
            }
            Err(_) => {
                metrics.failed_ops += 1;
            }
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

**Usage:**

```rust
let metrics = MetricsMiddleware::new();

let executor = FilesystemExecutor::default()
    .with_middleware(metrics.clone());

// ... perform operations ...

// Get statistics
let stats = metrics.get_stats().await;
println!("Total operations: {}", stats.total_ops);
println!("Success rate: {:.2}%", 
    (stats.successful_ops as f64 / stats.total_ops as f64) * 100.0);
```

#### Example 4: Retry Middleware (Conceptual)

```rust
#[derive(Debug, Clone)]
pub struct RetryMiddleware {
    max_attempts: u32,
    backoff_ms: u64,
}

impl RetryMiddleware {
    pub fn new(max_attempts: u32, backoff_ms: u64) -> Self {
        Self { max_attempts, backoff_ms }
    }
}

#[async_trait]
impl<O: Operation> Middleware<O> for RetryMiddleware {
    fn name(&self) -> &str {
        "retry"
    }
    
    fn priority(&self) -> u32 {
        25 // Low priority - retry failed operations
    }
    
    async fn can_process(&self, _operation: &O, _context: &ExecutionContext) -> bool {
        true
    }
    
    async fn before_execution(
        &self,
        operation: O,
        _context: &ExecutionContext,
    ) -> MiddlewareResult<Option<O>> {
        Ok(Some(operation))
    }
    
    async fn after_execution(
        &self,
        _context: &ExecutionContext,
        _result: &OSResult<ExecutionResult>,
    ) -> MiddlewareResult<()> {
        Ok(())
    }
    
    async fn handle_error(
        &self,
        error: OSError,
        _context: &ExecutionContext,
    ) -> ErrorAction {
        // Retry on transient errors
        match error {
            OSError::NetworkError { .. } | OSError::Timeout { .. } => {
                ErrorAction::Retry
            }
            _ => ErrorAction::Stop
        }
    }
}
```

### Testing Custom Middleware

#### Unit Testing

Test your middleware in isolation:

```rust
#[tokio::test]
async fn test_rate_limit_enforcement() {
    let limiter = RateLimitMiddleware::new(2); // 2 ops/sec
    
    // First two operations should succeed
    assert!(limiter.check_rate_limit("testuser").await);
    assert!(limiter.check_rate_limit("testuser").await);
    
    // Third operation should fail
    assert!(!limiter.check_rate_limit("testuser").await);
}

#[tokio::test]
async fn test_rate_limit_per_user() {
    let limiter = RateLimitMiddleware::new(1); // 1 op/sec
    
    // Different users have separate limits
    assert!(limiter.check_rate_limit("user1").await);
    assert!(limiter.check_rate_limit("user2").await);
    
    // Same user should be limited
    assert!(!limiter.check_rate_limit("user1").await);
}
```

#### Integration Testing

Test middleware with actual operations:

```rust
#[tokio::test]
async fn test_middleware_integration() {
    let limiter = RateLimitMiddleware::new(5);
    let executor = FilesystemExecutor::default()
        .with_middleware(limiter);
    
    let context = ExecutionContext::new(
        SecurityContext::new("test".to_string())
    );
    
    // Create a test file
    let temp_file = std::env::temp_dir().join("middleware_test.txt");
    std::fs::write(&temp_file, b"test data")
        .expect("Failed to create test file");
    
    // Operation should succeed
    let operation = FileReadOperation::new(
        temp_file.to_str().unwrap().to_string()
    );
    let result = executor.execute(operation, &context).await;
    
    // Cleanup
    let _ = std::fs::remove_file(&temp_file);
    
    assert!(result.is_ok(), "Operation should succeed within rate limit");
}
```

### Middleware Priority Guidelines

When setting priority values, follow these guidelines:

| Priority Range | Purpose | Examples |
|----------------|---------|----------|
| 90-100 | Critical security and validation | SecurityMiddleware (100) |
| 70-89 | Resource management | RateLimitMiddleware (75), CachingMiddleware (75) |
| 50-69 | Observability and metrics | MetricsMiddleware (50), LoggerMiddleware (50) |
| 25-49 | Error handling and recovery | RetryMiddleware (25) |
| 0-24 | Low-priority cross-cutting concerns | Custom audit trails, cleanup |

### Integration with Helper Functions

Custom middleware can be used with all three API levels:

**Level 1 - Simple Helpers (uses default middleware):**
```rust
// Cannot use custom middleware - uses defaults only
let data = read_file("/path", "user").await?;
```

**Level 2 - Custom Middleware Helpers:**
```rust
// Use with *_with_middleware variants
let custom = RateLimitMiddleware::new(100);
let data = read_file_with_middleware("/path", "user", custom).await?;
```

**Level 3 - Trait Composition (Future):**
```rust
// Build reusable pipelines
let helper = FileHelper::new()
    .with_middleware(RateLimitMiddleware::new(100))
    .with_middleware(MetricsMiddleware::new());

let data = helper.read("/path", "user").await?;
```

### Common Patterns

#### Pattern 1: Conditional Processing

```rust
async fn can_process(&self, operation: &O, context: &ExecutionContext) -> bool {
    // Only process operations from specific users
    context.security_context.principal == "admin"
}
```

#### Pattern 2: Operation Transformation

```rust
async fn before_execution(
    &self,
    mut operation: O,
    _context: &ExecutionContext,
) -> MiddlewareResult<Option<O>> {
    // Modify operation before execution
    // (requires mutable operation type)
    Ok(Some(operation))
}
```

#### Pattern 3: Short-Circuit Execution

```rust
async fn before_execution(
    &self,
    operation: O,
    _context: &ExecutionContext,
) -> MiddlewareResult<Option<O>> {
    if should_skip(&operation) {
        // Return None to skip executor
        return Ok(None);
    }
    Ok(Some(operation))
}
```

### Complete Working Example

For a complete, production-ready example of custom middleware, see:
- **[`examples/custom_middleware.rs`](../../../examples/custom_middleware.rs)** - Full RateLimitMiddleware implementation with tests

This example demonstrates:
- Thread-safe state management
- Sliding window rate limiting
- Integration with ExecutorExt
- Middleware chaining
- Helper function integration
- Comprehensive testing patterns

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

## Additional Examples

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
