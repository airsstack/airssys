//! Integration tests for multi-middleware chaining in composition API.
//!
//! Tests the ability to chain multiple middleware instances together
//! and verify their execution order.

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use std::fs;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use airssys_osl::core::context::ExecutionContext;
use airssys_osl::core::executor::ExecutionResult;
use airssys_osl::core::middleware::{Middleware, MiddlewareResult};
use airssys_osl::core::result::OSResult;
use airssys_osl::core::security::SecurityConfig;
use airssys_osl::helpers::composition::{FileHelper, HelperPipeline};
use airssys_osl::middleware::security::acl::{AccessControlList, AclEntry, AclPolicy};
use airssys_osl::middleware::security::middleware::SecurityMiddlewareBuilder;
use airssys_osl::operations::filesystem::FileReadOperation;

/// Custom middleware that logs execution to a shared buffer.
#[derive(Debug, Clone)]
struct LoggingMiddleware {
    name: String,
    log: Arc<Mutex<Vec<String>>>,
}

impl LoggingMiddleware {
    fn new(name: impl Into<String>, log: Arc<Mutex<Vec<String>>>) -> Self {
        Self {
            name: name.into(),
            log,
        }
    }
}

#[async_trait]
impl Middleware<FileReadOperation> for LoggingMiddleware {
    fn name(&self) -> &str {
        &self.name
    }

    async fn before_execution(
        &self,
        operation: FileReadOperation,
        _context: &ExecutionContext,
    ) -> MiddlewareResult<Option<FileReadOperation>> {
        // Log before execution
        self.log
            .lock()
            .unwrap()
            .push(format!("{} - before", self.name));

        // Pass through the operation unchanged
        Ok(Some(operation))
    }

    async fn after_execution(
        &self,
        _context: &ExecutionContext,
        _result: &OSResult<ExecutionResult>,
    ) -> MiddlewareResult<()> {
        // Log after execution
        self.log
            .lock()
            .unwrap()
            .push(format!("{} - after", self.name));

        // Success
        Ok(())
    }
}

/// Helper function to create a temporary test file
fn create_temp_file(content: &str) -> std::path::PathBuf {
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(format!("test_chaining_{}.txt", uuid::Uuid::new_v4()));
    fs::write(&file_path, content).expect("Failed to create temp file");
    file_path
}

/// Helper function to create permissive ACL
fn create_permissive_acl() -> AccessControlList {
    AccessControlList::new().add_entry(AclEntry::new(
        "testuser".to_string(),
        "*".to_string(),
        vec!["*".to_string()],
        AclPolicy::Allow,
    ))
}

#[tokio::test]
async fn test_security_with_custom_middleware() {
    let test_file = create_temp_file("Test content for chaining");
    let test_path = test_file.to_str().unwrap();

    // Create shared log
    let log = Arc::new(Mutex::new(Vec::new()));

    // Create SecurityMiddleware
    let acl = create_permissive_acl();
    let security = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    // Create custom logging middleware
    let logger = LoggingMiddleware::new("CustomLogger", Arc::clone(&log));

    // Create helper with both security and custom middleware
    let helper = FileHelper::builder()
        .with_security(security)
        .with_middleware(logger);

    // Execute operation
    let result = helper.read(test_path, "testuser").await;

    // Verify success
    assert!(
        result.is_ok(),
        "Operation should succeed with chained middleware"
    );

    // Verify custom middleware was executed
    let log_entries = log.lock().unwrap();
    assert!(
        log_entries.contains(&"CustomLogger - before".to_string()),
        "Custom middleware should execute before"
    );
    assert!(
        log_entries.contains(&"CustomLogger - after".to_string()),
        "Custom middleware should execute after"
    );

    // Cleanup
    fs::remove_file(test_file).ok();
}

#[tokio::test]
async fn test_multiple_custom_middleware() {
    let test_file = create_temp_file("Test content for multiple middleware");
    let test_path = test_file.to_str().unwrap();

    // Create shared log
    let log = Arc::new(Mutex::new(Vec::new()));

    // Create SecurityMiddleware
    let acl = create_permissive_acl();
    let security = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    // Create multiple custom middleware
    let logger1 = LoggingMiddleware::new("Logger1", Arc::clone(&log));
    let logger2 = LoggingMiddleware::new("Logger2", Arc::clone(&log));

    // Create helper with security and two custom middleware
    let helper = FileHelper::builder()
        .with_security(security)
        .with_middleware(logger1)
        .with_middleware(logger2);

    // Execute operation
    let result = helper.read(test_path, "testuser").await;

    // Verify success
    assert!(
        result.is_ok(),
        "Operation should succeed with multiple middleware"
    );

    // Verify both middleware were executed
    let log_entries = log.lock().unwrap();
    assert!(
        log_entries.contains(&"Logger1 - before".to_string()),
        "Logger1 should execute"
    );
    assert!(
        log_entries.contains(&"Logger2 - before".to_string()),
        "Logger2 should execute"
    );

    // Cleanup
    fs::remove_file(test_file).ok();
}

#[tokio::test]
async fn test_middleware_order_matters() {
    let test_file = create_temp_file("Test content for middleware order");
    let test_path = test_file.to_str().unwrap();

    // Create shared log
    let log = Arc::new(Mutex::new(Vec::new()));

    // Create SecurityMiddleware
    let acl = create_permissive_acl();
    let security = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    // Create middleware in specific order
    let logger_first = LoggingMiddleware::new("First", Arc::clone(&log));
    let logger_second = LoggingMiddleware::new("Second", Arc::clone(&log));
    let logger_third = LoggingMiddleware::new("Third", Arc::clone(&log));

    // Create helper with middleware in specific order
    let helper = FileHelper::builder()
        .with_security(security)
        .with_middleware(logger_first)
        .with_middleware(logger_second)
        .with_middleware(logger_third);

    // Execute operation
    let result = helper.read(test_path, "testuser").await;

    // Verify success
    assert!(result.is_ok(), "Operation should succeed");

    // Verify execution order (middleware execute in chain order for "before",
    // and reverse order for "after")
    let log_entries = log.lock().unwrap();

    // Debug: print all log entries to see what we captured
    println!("Log entries captured: {log_entries:?}");

    // Find positions of log entries
    let first_before_pos = log_entries
        .iter()
        .position(|e| e == "First - before")
        .expect("First - before should exist");
    let second_before_pos = log_entries
        .iter()
        .position(|e| e == "Second - before")
        .expect("Second - before should exist");
    let third_before_pos = log_entries
        .iter()
        .position(|e| e == "Third - before")
        .expect("Third - before should exist");

    // Verify "before" executes in REVERSE order (last added middleware runs first)
    // This is standard middleware onion pattern: Third → Second → First → [operation]
    assert!(
        third_before_pos < second_before_pos,
        "Third middleware (last added) should execute before Second"
    );
    assert!(
        second_before_pos < first_before_pos,
        "Second middleware should execute before First"
    );

    // Find "after" positions
    let first_after_pos = log_entries
        .iter()
        .position(|e| e == "First - after")
        .expect("First - after should exist");
    let second_after_pos = log_entries
        .iter()
        .position(|e| e == "Second - after")
        .expect("Second - after should exist");
    let third_after_pos = log_entries
        .iter()
        .position(|e| e == "Third - after")
        .expect("Third - after should exist");

    // Verify "after" executes in FORWARD order (unwinding the onion)
    // Order: [operation] → First → Second → Third
    assert!(
        first_after_pos < second_after_pos,
        "First middleware should complete after before Second"
    );
    assert!(
        second_after_pos < third_after_pos,
        "Second middleware should complete after before Third"
    );

    // Cleanup
    fs::remove_file(test_file).ok();
}
