//! Custom Middleware Example
//!
//! This example demonstrates how to create and use custom middleware with AirsSys OSL.
//!
//! # What This Example Shows
//!
//! 1. **Creating Custom Middleware**: Implement the `Middleware<O>` trait
//! 2. **Rate Limiting**: Track operations per second and deny if exceeded
//! 3. **Integration with ExecutorExt**: Use `.with_middleware()` pattern
//! 4. **Integration with Helpers**: Use `*_with_middleware` helper variants
//! 5. **Middleware Composition**: Chain multiple middleware together
//! 6. **Testing Patterns**: How to test custom middleware
//!
//! # Running This Example
//!
//! ```bash
//! cargo run --example custom_middleware
//! ```
//!
//! # Expected Output
//!
//! The example will:
//! - Create a rate-limited executor
//! - Perform file operations within rate limit (succeeds)
//! - Attempt to exceed rate limit (fails with error)
//! - Demonstrate middleware chaining
//! - Show helper function integration

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::sync::Arc;

// Layer 2: Third-party imports
use async_trait::async_trait;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

// Layer 3: Internal module imports
use airssys_osl::core::context::{ExecutionContext, SecurityContext};
use airssys_osl::core::executor::OSExecutor;
use airssys_osl::core::middleware::{ErrorAction, Middleware, MiddlewareError, MiddlewareResult};
use airssys_osl::core::operation::Operation;
use airssys_osl::core::result::{OSError, OSResult};
use airssys_osl::executors::filesystem::FilesystemExecutor;
use airssys_osl::helpers::{read_file_with_middleware, write_file_with_middleware};
use airssys_osl::middleware::ext::ExecutorExt;
use airssys_osl::middleware::logger::{ConsoleActivityLogger, LoggerMiddleware};
use airssys_osl::operations::filesystem::FileReadOperation;

// ============================================================================
// Custom Middleware Implementation: Rate Limiter
// ============================================================================

/// Rate limiting middleware that tracks operations per second.
///
/// This middleware demonstrates:
/// - Stateful middleware with Arc<Mutex<>> for thread-safe state management
/// - Time-based validation in `before_execution`
/// - Rejecting operations that exceed rate limits
/// - Error handling with descriptive messages
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::middleware::ext::ExecutorExt;
///
/// let rate_limiter = RateLimitMiddleware::new(10); // 10 ops/sec
/// let executor = FilesystemExecutor::default()
///     .with_middleware(rate_limiter);
/// ```
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

impl RateLimitMiddleware {
    /// Create a new rate limiter with specified operations per second limit.
    ///
    /// # Arguments
    ///
    /// * `max_ops_per_second` - Maximum number of operations allowed per user per second
    ///
    /// # Example
    ///
    /// ```rust
    /// # use custom_middleware::RateLimitMiddleware;
    /// // Allow 100 operations per second per user
    /// let limiter = RateLimitMiddleware::new(100);
    /// ```
    pub fn new(max_ops_per_second: u32) -> Self {
        Self {
            max_ops_per_second,
            state: Arc::new(Mutex::new(RateLimitState {
                operation_times: HashMap::new(),
            })),
        }
    }

    /// Check if the user has exceeded their rate limit.
    ///
    /// This method:
    /// 1. Gets current timestamp
    /// 2. Retrieves user's recent operation times
    /// 3. Filters out operations older than 1 second
    /// 4. Counts remaining operations in the current window
    /// 5. Returns true if under limit, false if exceeded
    async fn check_rate_limit(&self, user: &str) -> bool {
        let mut state = self.state.lock().await;
        let now = Instant::now();
        let one_second_ago = now - Duration::from_secs(1);

        // Get or create user's operation history
        let times = state
            .operation_times
            .entry(user.to_string())
            .or_insert_with(Vec::new);

        // Remove operations older than 1 second
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
        // No post-processing needed
        Ok(())
    }

    async fn handle_error(&self, _error: OSError, _context: &ExecutionContext) -> ErrorAction {
        // Let rate limit errors propagate
        ErrorAction::Stop
    }
}

// ============================================================================
// Example Usage Functions
// ============================================================================

/// Demonstrates basic rate limiting with executor + middleware pattern.
async fn example_basic_rate_limiting() -> OSResult<()> {
    println!("\n=== Example 1: Basic Rate Limiting ===\n");

    // Create rate limiter: 2 operations per second
    let rate_limiter = RateLimitMiddleware::new(2);

    // Wrap executor with rate limiting middleware
    let executor = FilesystemExecutor::default().with_middleware(rate_limiter);

    // Create context
    let context = ExecutionContext::new(SecurityContext::new("testuser".to_string()));

    println!("Rate limit: 2 operations per second");
    println!("Attempting 3 operations in rapid succession...\n");

    // Operation 1: Should succeed
    let op1 = FileReadOperation::new("/tmp/test1.txt".to_string());
    match executor.execute(op1, &context).await {
        Ok(_) => println!("✓ Operation 1: SUCCESS (within rate limit)"),
        Err(e) => println!("✗ Operation 1: FAILED - {}", e),
    }

    // Operation 2: Should succeed
    let op2 = FileReadOperation::new("/tmp/test2.txt".to_string());
    match executor.execute(op2, &context).await {
        Ok(_) => println!("✓ Operation 2: SUCCESS (within rate limit)"),
        Err(e) => println!("✗ Operation 2: FAILED - {}", e),
    }

    // Operation 3: Should fail (rate limit exceeded)
    let op3 = FileReadOperation::new("/tmp/test3.txt".to_string());
    match executor.execute(op3, &context).await {
        Ok(_) => println!("✓ Operation 3: SUCCESS (unexpected!)"),
        Err(e) => println!("✗ Operation 3: FAILED - {} (expected!)", e),
    }

    println!("\nWaiting 1 second for rate limit window to reset...");
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Operation 4: Should succeed (new window)
    let op4 = FileReadOperation::new("/tmp/test4.txt".to_string());
    match executor.execute(op4, &context).await {
        Ok(_) => println!("✓ Operation 4: SUCCESS (new rate limit window)"),
        Err(e) => println!("✗ Operation 4: FAILED - {}", e),
    }

    Ok(())
}

/// Demonstrates middleware chaining with multiple custom middleware.
async fn example_middleware_chaining() -> OSResult<()> {
    println!("\n=== Example 2: Middleware Chaining ===\n");

    // Create multiple middleware
    let rate_limiter = RateLimitMiddleware::new(5); // 5 ops/sec
    let logger = LoggerMiddleware::with_default_config(ConsoleActivityLogger::default());

    // Chain middleware: logger wraps rate_limiter wraps executor
    let executor = FilesystemExecutor::default()
        .with_middleware(rate_limiter)
        .with_middleware(logger);

    let context = ExecutionContext::new(SecurityContext::new("admin".to_string()));

    println!("Chained middleware: Logger -> RateLimiter -> Executor");
    println!("Performing operation...\n");

    let operation = FileReadOperation::new("/tmp/chained.txt".to_string());
    match executor.execute(operation, &context).await {
        Ok(_) => println!("\n✓ Chained operation succeeded"),
        Err(e) => println!("\n✗ Chained operation failed: {}", e),
    }

    Ok(())
}

/// Demonstrates using custom middleware with helper functions.
async fn example_helper_integration() -> OSResult<()> {
    println!("\n=== Example 3: Helper Function Integration ===\n");

    // Create rate limiter
    let rate_limiter = RateLimitMiddleware::new(3); // 3 ops/sec

    println!("Using helper function with custom middleware");
    println!("Rate limit: 3 operations per second\n");

    // Use helper function with custom middleware
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("helper_test.txt");

    // Write operation (counts toward rate limit)
    match write_file_with_middleware(
        test_file.to_str().unwrap(),
        b"Test data".to_vec(),
        "helper_user",
        rate_limiter.clone(),
    )
    .await
    {
        Ok(_) => println!("✓ Write operation: SUCCESS"),
        Err(e) => println!("✗ Write operation: FAILED - {}", e),
    }

    // Read operation (counts toward rate limit)
    match read_file_with_middleware(
        test_file.to_str().unwrap(),
        "helper_user",
        rate_limiter.clone(),
    )
    .await
    {
        Ok(data) => println!("✓ Read operation: SUCCESS (read {} bytes)", data.len()),
        Err(e) => println!("✗ Read operation: FAILED - {}", e),
    }

    // Cleanup
    let _ = std::fs::remove_file(&test_file);

    Ok(())
}

/// Demonstrates testing patterns for custom middleware.
#[cfg(test)]
mod tests {
    use super::*;

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

    #[tokio::test]
    async fn test_rate_limit_window_reset() {
        let limiter = RateLimitMiddleware::new(1); // 1 op/sec

        // Use up the quota
        assert!(limiter.check_rate_limit("testuser").await);
        assert!(!limiter.check_rate_limit("testuser").await);

        // Wait for window to reset
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Should succeed again
        assert!(limiter.check_rate_limit("testuser").await);
    }

    #[tokio::test]
    async fn test_middleware_integration() {
        let limiter = RateLimitMiddleware::new(5);
        let executor = FilesystemExecutor::default().with_middleware(limiter);
        let context = ExecutionContext::new(SecurityContext::new("test".to_string()));

        // Create a temporary file for testing
        let temp_file = std::env::temp_dir().join("middleware_test.txt");
        std::fs::write(&temp_file, b"test data").expect("Failed to create test file");

        // Operation should succeed
        let operation = FileReadOperation::new(temp_file.to_str().unwrap().to_string());
        let result = executor.execute(operation, &context).await;

        // Cleanup
        let _ = std::fs::remove_file(&temp_file);

        assert!(result.is_ok(), "Operation should succeed within rate limit");
    }
}

// ============================================================================
// Main Function
// ============================================================================

#[tokio::main]
async fn main() -> OSResult<()> {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║           AirsSys OSL - Custom Middleware Example             ║");
    println!("╚════════════════════════════════════════════════════════════════╝");

    // Run all examples
    example_basic_rate_limiting().await?;
    example_middleware_chaining().await?;
    example_helper_integration().await?;

    println!("\n=== Summary ===\n");
    println!("This example demonstrated:");
    println!("  ✓ Creating custom middleware (RateLimitMiddleware)");
    println!("  ✓ Implementing Middleware<O> trait");
    println!("  ✓ Using with ExecutorExt (.with_middleware)");
    println!("  ✓ Chaining multiple middleware");
    println!("  ✓ Integration with helper functions");
    println!("  ✓ Testing patterns (see tests module)");
    println!();
    println!("Next steps:");
    println!("  - Review the source code in examples/custom_middleware.rs");
    println!("  - Run tests: cargo test --example custom_middleware");
    println!("  - See guides/middleware.md for more patterns");
    println!();

    Ok(())
}
