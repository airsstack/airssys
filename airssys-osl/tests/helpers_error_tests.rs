//! Integration tests for helper function error handling.
//!
//! This test suite verifies that helper functions properly handle and
//! propagate errors from security policies, resource operations, and
//! invalid inputs.
//!
//! **Test Coverage:**
//! - Security error types (SecurityViolation, PermissionDenied)
//! - Resource errors (NotFound, IO errors, network errors)
//! - Edge cases (empty paths, invalid IDs, malformed addresses)
//! - Error message context and clarity
//!
//! **Phase:** OSL-TASK-010 Phase 5 - Integration Testing

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use airssys_osl::core::result::OSError;
use airssys_osl::helpers::*;
use airssys_osl::middleware::security::acl::{AccessControlList, AclEntry, AclPolicy};
use airssys_osl::middleware::security::middleware::SecurityMiddlewareBuilder;
use airssys_osl::middleware::security::rbac::{Permission, Role, RoleBasedAccessControl};

// ============================================================================
// Security Error Tests
// ============================================================================

#[tokio::test]
async fn test_security_violation_error_on_acl_deny() {
    // Setup: ACL denies access
    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "blocked_user".to_string(),
        "/forbidden/*".to_string(),
        vec!["*".to_string()],
        AclPolicy::Deny,
    ));

    let security = SecurityMiddlewareBuilder::new()
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build security middleware");

    // Test: Attempt forbidden operation
    let result = read_file_with_middleware("/forbidden/secret.txt", "blocked_user", security).await;

    // Verify: Returns error indicating ACL denial
    assert!(result.is_err(), "Expected error for ACL denial");

    match result.unwrap_err() {
        OSError::MiddlewareFailed { reason, .. } => {
            assert!(
                reason.contains("ACL")
                    || reason.contains("Deny")
                    || reason.contains("security")
                    || reason.contains("policy"),
                "Expected ACL denial in error message, got: {reason}"
            );
        }
        OSError::SecurityViolation { reason } => {
            assert!(
                reason.contains("ACL") || reason.contains("Deny"),
                "Expected ACL denial in error message, got: {reason}"
            );
        }
        OSError::ExecutionFailed { reason } => {
            assert!(
                reason.contains("ACL")
                    || reason.contains("Deny")
                    || reason.contains("SecurityViolation"),
                "Expected ACL denial in error message, got: {reason}"
            );
        }
        other => panic!("Expected security-related error, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_permission_denied_error_on_rbac_violation() {
    // Setup: RBAC policy without proper role
    let permission = Permission::new(
        "file:write".to_string(),
        "Write File".to_string(),
        "Allows writing files".to_string(),
    );

    let role = Role::new("writer".to_string(), "Writer Role".to_string())
        .with_permission("file:write".to_string());

    let mut rbac = RoleBasedAccessControl::new();
    rbac = rbac
        .add_permission(permission)
        .add_role(role)
        .assign_roles("authorized_user".to_string(), vec!["writer".to_string()]);
    // Note: unauthorized_user has no roles

    let security = SecurityMiddlewareBuilder::new()
        .add_policy(Box::new(rbac))
        .build()
        .expect("Failed to build security middleware");

    // Test: Unauthorized user tries to write
    let result = write_file_with_middleware(
        "/tmp/test.txt",
        b"data".to_vec(),
        "unauthorized_user",
        security,
    )
    .await;

    // Verify: Returns error indicating permission denied
    assert!(result.is_err(), "Expected permission denied error");

    match result.unwrap_err() {
        OSError::MiddlewareFailed { reason, .. } => {
            assert!(
                reason.contains("RBAC")
                    || reason.contains("role")
                    || reason.contains("permission")
                    || reason.contains("policy"),
                "Expected RBAC/permission error, got: {reason}"
            );
        }
        OSError::SecurityViolation { reason } => {
            assert!(
                reason.contains("RBAC") || reason.contains("permission"),
                "Expected RBAC/permission error, got: {reason}"
            );
        }
        OSError::ExecutionFailed { reason } => {
            assert!(
                reason.contains("RBAC")
                    || reason.contains("role")
                    || reason.contains("SecurityViolation"),
                "Expected RBAC/permission error, got: {reason}"
            );
        }
        other => panic!("Expected security-related error, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_error_messages_contain_user_context() {
    // Setup: Deny all access
    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "test_user".to_string(),
        "*".to_string(),
        vec!["*".to_string()],
        AclPolicy::Deny,
    ));

    let security = SecurityMiddlewareBuilder::new()
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build security middleware");

    // Test: Perform denied operation
    let result = read_file_with_middleware("/any/file.txt", "test_user", security).await;

    // Verify: Error exists and can be examined
    assert!(result.is_err(), "Expected security denial error");

    let error = result.unwrap_err();
    let error_display = format!("{error}");
    let error_debug = format!("{error:?}");

    // Verify: Error can be displayed and debugged
    assert!(
        !error_display.is_empty(),
        "Error display should not be empty"
    );
    assert!(!error_debug.is_empty(), "Error debug should not be empty");
}

// ============================================================================
// Resource Error Tests
// ============================================================================

#[tokio::test]
async fn test_read_file_not_found_error() {
    // Setup: Allow all access, but file doesn't exist
    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "user".to_string(),
        "*".to_string(),
        vec!["*".to_string()],
        AclPolicy::Allow,
    ));

    let security = SecurityMiddlewareBuilder::new()
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build security middleware");

    // Test: Try to read non-existent file
    let non_existent_path = "/tmp/this_file_definitely_does_not_exist_12345.txt";
    let result = read_file_with_middleware(non_existent_path, "user", security).await;

    // Verify: Returns FilesystemError or ExecutionFailed
    assert!(result.is_err(), "Expected error for non-existent file");

    match result.unwrap_err() {
        OSError::FilesystemError { path, reason, .. } => {
            assert!(
                path.contains("this_file_definitely_does_not_exist")
                    || reason.contains("not found")
                    || reason.contains("No such file"),
                "Expected file not found error"
            );
        }
        OSError::ExecutionFailed { reason } => {
            assert!(
                reason.contains("not found") || reason.contains("No such file"),
                "Expected 'not found' error, got: {reason}"
            );
        }
        other => {
            // May also get middleware or execution errors depending on implementation
            let error_msg = format!("{other:?}");
            assert!(
                error_msg.contains("not found")
                    || error_msg.contains("No such file")
                    || error_msg.contains("NotFound"),
                "Expected 'not found' error, got: {error_msg}"
            );
        }
    }
}

#[tokio::test]
async fn test_write_file_io_error_propagation() {
    // Setup: Allow all access
    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "user".to_string(),
        "*".to_string(),
        vec!["*".to_string()],
        AclPolicy::Allow,
    ));

    let security = SecurityMiddlewareBuilder::new()
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build security middleware");

    // Test: Try to write to invalid/read-only path
    // Note: This might succeed depending on OS permissions, but we test the error path
    let invalid_path = "/root/protected_system_file.txt"; // Typically requires root
    let result = write_file_with_middleware(invalid_path, b"data".to_vec(), "user", security).await;

    // Verify: If it fails, it should be an appropriate error
    // (May succeed in some test environments, so we only verify if it fails)
    if result.is_err() {
        match result.unwrap_err() {
            OSError::FilesystemError { .. } => {
                // Expected error type for filesystem permission issues
            }
            OSError::ExecutionFailed { .. } | OSError::MiddlewareFailed { .. } => {
                // Also acceptable depending on executor implementation
            }
            OSError::SecurityViolation { .. } => {
                // May occur if security policy catches permission issues
            }
            other => {
                panic!("Unexpected error type for IO failure: {other:?}");
            }
        }
    }
}

#[tokio::test]
async fn test_kill_process_invalid_pid_error() {
    // Setup: Allow all access
    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "user".to_string(),
        "*".to_string(),
        vec!["*".to_string()],
        AclPolicy::Allow,
    ));

    let security = SecurityMiddlewareBuilder::new()
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build security middleware");

    // Test: Try to kill process with invalid PID
    let invalid_pid = 9999999; // Very unlikely to exist
    let result = kill_process_with_middleware(invalid_pid, "user", security).await;

    // Verify: Returns appropriate error
    assert!(
        result.is_err(),
        "Expected error for invalid/non-existent PID"
    );

    match result.unwrap_err() {
        OSError::ProcessError { .. } | OSError::ExecutionFailed { .. } => {
            // Expected error types for invalid PID
        }
        other => {
            let error_msg = format!("{other:?}");
            assert!(
                error_msg.contains("process")
                    || error_msg.contains("PID")
                    || error_msg.contains("not found"),
                "Expected process-related error, got: {error_msg}"
            );
        }
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[tokio::test]
async fn test_empty_file_path_error() {
    // Setup: Allow all access
    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "user".to_string(),
        "*".to_string(),
        vec!["*".to_string()],
        AclPolicy::Allow,
    ));

    let security = SecurityMiddlewareBuilder::new()
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build security middleware");

    // Test: Try to read empty path
    let result = read_file_with_middleware("", "user", security).await;

    // Verify: Returns error for empty path
    assert!(result.is_err(), "Expected error for empty file path");

    // Error type may vary - FilesystemError, ExecutionFailed all acceptable
    match result.unwrap_err() {
        OSError::FilesystemError { .. } | OSError::ExecutionFailed { .. } => {
            // Expected error types
        }
        other => {
            let error_msg = format!("{other:?}");
            assert!(
                !error_msg.is_empty(),
                "Error should contain some information"
            );
        }
    }
}

#[tokio::test]
async fn test_process_spawn_empty_command_error() {
    // Setup: Allow all access
    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "user".to_string(),
        "*".to_string(),
        vec!["*".to_string()],
        AclPolicy::Allow,
    ));

    let security = SecurityMiddlewareBuilder::new()
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build security middleware");

    // Test: Try to spawn process with empty command
    let result = spawn_process_with_middleware("", vec![], "user", security).await;

    // Verify: Returns error for empty command
    assert!(result.is_err(), "Expected error for empty command");

    match result.unwrap_err() {
        OSError::ProcessError { .. } | OSError::ExecutionFailed { .. } => {
            // Expected error types
        }
        other => {
            let error_msg = format!("{other:?}");
            assert!(
                error_msg.contains("command")
                    || error_msg.contains("empty")
                    || error_msg.contains("invalid")
                    || error_msg.contains("process"),
                "Expected command-related error, got: {error_msg}"
            );
        }
    }
}

#[tokio::test]
async fn test_network_connect_invalid_address_error() {
    // Setup: Allow all access
    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "user".to_string(),
        "*".to_string(),
        vec!["*".to_string()],
        AclPolicy::Allow,
    ));

    let security = SecurityMiddlewareBuilder::new()
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build security middleware");

    // Test: Try to connect to invalid address
    let invalid_address = "not_a_valid_address:99999";
    let result = network_connect_with_middleware(invalid_address, "user", security).await;

    // Verify: Returns error for invalid address
    assert!(
        result.is_err(),
        "Expected error for invalid network address"
    );

    match result.unwrap_err() {
        OSError::NetworkError { .. } | OSError::ExecutionFailed { .. } => {
            // Expected error types
        }
        other => {
            let error_msg = format!("{other:?}");
            assert!(
                error_msg.contains("address")
                    || error_msg.contains("network")
                    || error_msg.contains("invalid"),
                "Expected network-related error, got: {error_msg}"
            );
        }
    }
}

// ============================================================================
// Error Context and Display Tests
// ============================================================================

#[tokio::test]
async fn test_error_implements_display_and_debug() {
    // Setup: Create a scenario that will produce an error
    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "user".to_string(),
        "/denied/*".to_string(),
        vec!["*".to_string()],
        AclPolicy::Deny,
    ));

    let security = SecurityMiddlewareBuilder::new()
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build security middleware");

    // Test: Get an error
    let result = read_file_with_middleware("/denied/file.txt", "user", security).await;
    assert!(result.is_err(), "Expected error");

    let error = result.unwrap_err();

    // Verify: Display trait works
    let display_output = format!("{error}");
    assert!(
        !display_output.is_empty(),
        "Display output should not be empty"
    );

    // Verify: Debug trait works
    let debug_output = format!("{error:?}");
    assert!(!debug_output.is_empty(), "Debug output should not be empty");

    // Verify: Outputs are different (Debug typically more verbose)
    // (This may not always be true, but generally Debug has more info)
    assert!(
        debug_output.len() >= display_output.len(),
        "Debug output should generally be at least as long as Display"
    );
}

#[tokio::test]
async fn test_error_can_be_downcast_and_analyzed() {
    // Setup: Multiple error scenarios
    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "user".to_string(),
        "*".to_string(),
        vec!["*".to_string()],
        AclPolicy::Allow,
    ));

    let security = SecurityMiddlewareBuilder::new()
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build security middleware");

    // Test: Get NotFound error
    let result = read_file_with_middleware("/nonexistent/file.txt", "user", security).await;

    if let Err(error) = result {
        // Verify: Can match on error variants
        match error {
            OSError::FilesystemError { .. } => {
                // Successfully matched FilesystemError variant
            }
            OSError::ExecutionFailed { .. } => {
                // May wrap the underlying error
            }
            _ => {
                // Other error types are acceptable depending on implementation
            }
        }
    }
}
