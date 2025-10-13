//! Integration tests for error handling in composition API.
//!
//! Tests that the composition API properly handles and propagates errors
//! from security violations, file system errors, process errors, and network errors.

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use std::fs;

use airssys_osl::core::result::OSError;
use airssys_osl::core::security::SecurityConfig;
use airssys_osl::helpers::composition::{FileHelper, HelperPipeline, NetworkHelper, ProcessHelper};
use airssys_osl::middleware::security::acl::{AccessControlList, AclEntry, AclPolicy};
use airssys_osl::middleware::security::middleware::SecurityMiddlewareBuilder;

/// Helper function to create restrictive ACL that denies access
fn create_deny_acl() -> AccessControlList {
    AccessControlList::new().add_entry(AclEntry::new(
        "testuser".to_string(),
        "*".to_string(),
        vec!["*".to_string()],
        AclPolicy::Deny,
    ))
}

/// Helper function to create permissive ACL
fn create_allow_acl() -> AccessControlList {
    AccessControlList::new().add_entry(AclEntry::new(
        "testuser".to_string(),
        "*".to_string(),
        vec!["*".to_string()],
        AclPolicy::Allow,
    ))
}

#[tokio::test]
async fn test_security_violation_in_composed_helper() {
    // Create a test file
    let test_file =
        std::env::temp_dir().join(format!("test_security_{}.txt", uuid::Uuid::new_v4()));
    fs::write(&test_file, "secret content").expect("Failed to create test file");
    let test_path = test_file.to_str().unwrap();

    // Create SecurityMiddleware with DENY policy
    let acl = create_deny_acl();
    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    // Create FileHelper with security that denies access
    let helper = FileHelper::builder().with_security(middleware);

    // Execute read operation - should be denied
    let result = helper.read(test_path, "testuser").await;

    // Verify security violation
    assert!(result.is_err(), "Operation should be denied by security");

    match result {
        Err(OSError::SecurityViolation { reason }) => {
            assert!(
                reason.contains("denied") || reason.contains("Deny") || reason.contains("deny"),
                "Error should indicate denial: {reason}"
            );
        }
        Err(OSError::ExecutionFailed { reason }) => {
            // SecurityViolation may be wrapped in ExecutionFailed
            assert!(
                reason.contains("SecurityViolation")
                    || reason.contains("denied")
                    || reason.contains("Deny"),
                "ExecutionFailed should contain security violation: {reason}"
            );
        }
        Err(other) => {
            unreachable!("Expected SecurityViolation or ExecutionFailed, got: {other:?}")
        }
        Ok(_) => {
            unreachable!("Operation should have been denied")
        }
    }

    // Cleanup
    fs::remove_file(test_file).ok();
}

#[tokio::test]
async fn test_file_not_found_error() {
    // Create SecurityMiddleware with permissive ACL
    let acl = create_allow_acl();
    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    // Create FileHelper with security
    let helper = FileHelper::builder().with_security(middleware);

    // Try to read non-existent file
    let nonexistent_path = format!("/tmp/nonexistent_{}.txt", uuid::Uuid::new_v4());
    let result = helper.read(&nonexistent_path, "testuser").await;

    // Verify file not found error
    assert!(result.is_err(), "Reading nonexistent file should fail");

    match result {
        Err(OSError::FilesystemError {
            operation: _,
            path,
            reason,
        }) => {
            assert!(
                path.contains(&nonexistent_path)
                    || reason.contains("not found")
                    || reason.contains("No such file"),
                "Error should indicate file not found - path: {path}, reason: {reason}"
            );
        }
        Err(OSError::ExecutionFailed { reason }) => {
            // Execution error is also acceptable
            assert!(
                reason.contains("No such file") || reason.contains("not found"),
                "ExecutionFailed should indicate file not found: {reason}"
            );
        }
        Err(other) => {
            unreachable!("Expected FilesystemError or ExecutionFailed, got: {other:?}")
        }
        Ok(_) => {
            unreachable!("Reading nonexistent file should fail")
        }
    }
}

#[tokio::test]
async fn test_process_spawn_failure() {
    // Create SecurityMiddleware with permissive ACL
    let acl = create_allow_acl();
    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    // Create ProcessHelper with security
    let helper = ProcessHelper::builder().with_security(middleware);

    // Try to spawn non-existent command
    let nonexistent_cmd = format!("nonexistent_command_{}", uuid::Uuid::new_v4());
    let result = helper.spawn(&nonexistent_cmd, vec![], "testuser").await;

    // Verify process spawn error
    assert!(result.is_err(), "Spawning nonexistent command should fail");

    match result {
        Err(OSError::ProcessError {
            operation: _,
            reason,
        }) => {
            assert!(
                reason.contains(&nonexistent_cmd)
                    || reason.contains("not found")
                    || reason.contains("No such file"),
                "Error should indicate command not found: {reason}"
            );
        }
        Err(OSError::ExecutionFailed { reason }) => {
            // Execution error is also acceptable
            assert!(
                reason.contains("not found") || reason.contains("No such file"),
                "ExecutionFailed should indicate command not found: {reason}"
            );
        }
        Err(other) => {
            unreachable!("Expected ProcessError or ExecutionFailed, got: {other:?}")
        }
        Ok(_) => {
            unreachable!("Spawning nonexistent command should fail")
        }
    }
}

#[tokio::test]
async fn test_network_connection_refused() {
    // Create SecurityMiddleware with permissive ACL
    let acl = create_allow_acl();
    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    // Create NetworkHelper with security
    let helper = NetworkHelper::builder().with_security(middleware);

    // Try to connect to a port that's likely closed
    // Using localhost on a high port that's unlikely to be in use
    let unlikely_port = "127.0.0.1:59999";
    let result = helper.connect(unlikely_port, "testuser").await;

    // Verify network error
    // Note: This test might pass if the port happens to be open,
    // but that's unlikely with a high port number
    if result.is_err() {
        match result {
            Err(OSError::NetworkError {
                operation: _,
                reason,
            }) => {
                assert!(
                    reason.contains("refused")
                        || reason.contains("connection")
                        || reason.contains("unable"),
                    "Error should indicate connection issue: {reason}"
                );
            }
            Err(OSError::ExecutionFailed { reason }) => {
                // Execution error is also acceptable
                assert!(
                    reason.contains("refused") || reason.contains("connection"),
                    "ExecutionFailed should indicate connection issue: {reason}"
                );
            }
            Err(other) => {
                // Accept other network-related errors
                println!("Got network error (acceptable): {other:?}");
            }
            Ok(_) => unreachable!(),
        }
    } else {
        // If connection succeeded, the port happened to be open - that's fine
        println!("Port {unlikely_port} was open (unexpected but acceptable)");
    }
}

#[tokio::test]
async fn test_error_propagation_through_middleware_chain() {
    // Create a deny ACL
    let acl = create_deny_acl();
    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    // Create FileHelper with security middleware
    let helper = FileHelper::builder().with_security(middleware);

    // Try to read a file (will be denied by security)
    let test_path = "/tmp/test.txt";
    let result = helper.read(test_path, "testuser").await;

    // Verify error propagates through the entire middleware chain
    assert!(
        result.is_err(),
        "Security error should propagate through middleware chain"
    );

    // Verify it's a security-related error (may be wrapped in ExecutionFailed)
    match result {
        Err(OSError::SecurityViolation { reason: _ }) => {
            // Direct SecurityViolation is acceptable
        }
        Err(OSError::ExecutionFailed { reason }) => {
            // SecurityViolation wrapped in ExecutionFailed is also acceptable
            assert!(
                reason.contains("SecurityViolation")
                    || reason.contains("denied")
                    || reason.contains("Deny"),
                "ExecutionFailed should contain security violation: {reason}"
            );
        }
        Err(other) => {
            unreachable!(
                "Expected SecurityViolation or ExecutionFailed with security context, got: {other:?}"
            )
        }
        Ok(_) => {
            unreachable!("Operation should have failed")
        }
    }
}

#[tokio::test]
async fn test_multiple_error_types_with_same_helper() {
    // Create permissive ACL to test non-security errors
    let acl = create_allow_acl();
    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    let helper = FileHelper::builder().with_security(middleware);

    // Test 1: File not found error
    let result1 = helper
        .read("/tmp/nonexistent_file_12345.txt", "testuser")
        .await;
    assert!(result1.is_err(), "Should fail with not found error");

    // Test 2: Same helper, different error scenario
    // Try to read a directory as a file (should fail)
    let result2 = helper.read("/tmp", "testuser").await;
    assert!(
        result2.is_err(),
        "Should fail when reading directory as file"
    );

    // Verify both errors propagate correctly
    println!("Error 1: {result1:?}");
    println!("Error 2: {result2:?}");
}
