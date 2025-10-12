//! Integration tests for helper function audit logging.
//!
//! This test suite verifies that all helper functions properly generate
//! audit logs through the SecurityMiddleware integration.
//!
//! **Test Coverage:**
//! - Successful operations logged with user context
//! - Failed operations logged with denial reasons
//! - Operation metadata captured correctly
//! - Security violations logged comprehensively
//!
//! **Phase:** OSL-TASK-010 Phase 5 - Integration Testing

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use airssys_osl::helpers::*;
use airssys_osl::middleware::security::acl::{AccessControlList, AclEntry, AclPolicy};
use airssys_osl::middleware::security::audit::{AuditError, SecurityAuditLog, SecurityAuditLogger};
use airssys_osl::middleware::security::middleware::SecurityMiddlewareBuilder;
use airssys_osl::middleware::security::rbac::{Permission, Role, RoleBasedAccessControl};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

// ============================================================================
// Test Audit Logger Implementation
// ============================================================================

/// Test audit logger that captures events in memory for verification.
#[derive(Debug, Clone, Default)]
pub struct TestSecurityAuditLogger {
    events: Arc<Mutex<Vec<SecurityAuditLog>>>,
}

impl TestSecurityAuditLogger {
    /// Create a new test audit logger.
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get all captured audit events.
    pub fn get_events(&self) -> Vec<SecurityAuditLog> {
        self.events.lock().unwrap().clone()
    }

    /// Get the number of captured events.
    pub fn event_count(&self) -> usize {
        self.events.lock().unwrap().len()
    }

    /// Clear all captured events.
    pub fn clear(&self) {
        self.events.lock().unwrap().clear();
    }

    /// Check if any event matches a predicate.
    pub fn has_event<F>(&self, predicate: F) -> bool
    where
        F: Fn(&SecurityAuditLog) -> bool,
    {
        self.events.lock().unwrap().iter().any(predicate)
    }
}

#[async_trait]
impl SecurityAuditLogger for TestSecurityAuditLogger {
    async fn log_security_event(&self, event: SecurityAuditLog) -> Result<(), AuditError> {
        self.events.lock().unwrap().push(event);
        Ok(())
    }
}

// ============================================================================
// Successful Operation Logging Tests
// ============================================================================

#[tokio::test]
async fn test_read_file_successful_operation_logged() {
    // Setup: Create security middleware with test logger
    let test_logger = Arc::new(TestSecurityAuditLogger::new());

    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "admin".to_string(),
        "/tmp/*".to_string(),
        vec!["read".to_string()],
        AclPolicy::Allow,
    ));

    let security = SecurityMiddlewareBuilder::new()
        .add_policy(Box::new(acl))
        .with_audit_logger(test_logger.clone())
        .build()
        .expect("Failed to build security middleware");

    // Create test file
    let test_path = "/tmp/test_audit_read.txt";
    std::fs::write(test_path, b"audit test data").expect("Failed to create test file");

    // Test: Perform read operation
    let result = read_file_with_middleware(test_path, "admin", security).await;

    // Cleanup
    let _ = std::fs::remove_file(test_path);

    // Verify: Operation succeeded
    assert!(result.is_ok(), "Expected read to succeed");

    // Verify: Audit log was created
    let events = test_logger.get_events();
    assert!(
        !events.is_empty(),
        "Expected at least one audit event, got: {}",
        events.len()
    );

    // Verify: Event contains correct user context
    let has_correct_user = events.iter().any(|e| e.principal == "admin");
    assert!(
        has_correct_user,
        "Expected audit log with principal 'admin', got events: {:?}",
        events
    );

    // Verify: Event indicates access was granted
    let has_allow_decision = events.iter().any(|e| e.decision.contains("Allow"));
    assert!(
        has_allow_decision,
        "Expected 'Allow' decision in audit log, got events: {:?}",
        events
    );
}

#[tokio::test]
async fn test_spawn_process_successful_operation_logged() {
    // Setup: Create security middleware with test logger
    let test_logger = Arc::new(TestSecurityAuditLogger::new());

    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "operator".to_string(),
        "process".to_string(), // ProcessSpawn operations use "process" as resource
        vec!["spawn".to_string()],
        AclPolicy::Allow,
    ));

    // Add RBAC policy with process:spawn permission
    let permission = Permission::new(
        "process:spawn".to_string(),
        "Spawn Process".to_string(),
        "Allows spawning processes".to_string(),
    );

    let role = Role::new("operator_role".to_string(), "Operator Role".to_string())
        .with_permission("process:spawn".to_string());

    let rbac = RoleBasedAccessControl::new()
        .add_permission(permission)
        .add_role(role)
        .assign_roles("operator".to_string(), vec!["operator_role".to_string()]);

    let security = SecurityMiddlewareBuilder::new()
        .add_policy(Box::new(acl))
        .add_policy(Box::new(rbac))
        .with_audit_logger(test_logger.clone())
        .build()
        .expect("Failed to build security middleware");

    // Test: Spawn a process
    let result =
        spawn_process_with_middleware("echo", vec!["audit test".to_string()], "operator", security)
            .await;

    // Verify: Operation succeeded
    if let Err(e) = &result {
        eprintln!("Spawn process error: {:?}", e);
    }
    assert!(result.is_ok(), "Expected spawn to succeed");

    // Verify: Audit log was created
    let events = test_logger.get_events();
    assert!(!events.is_empty(), "Expected audit event for process spawn");

    // Verify: Event contains operator user
    let has_operator = events.iter().any(|e| e.principal == "operator");
    assert!(
        has_operator,
        "Expected audit log for 'operator' user, got: {:?}",
        events
    );
}

#[tokio::test]
async fn test_write_file_operation_metadata_captured() {
    // Setup: Create security middleware with test logger
    let test_logger = Arc::new(TestSecurityAuditLogger::new());

    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "alice".to_string(),
        "/tmp/*".to_string(),
        vec!["write".to_string()],
        AclPolicy::Allow,
    ));

    let security = SecurityMiddlewareBuilder::new()
        .add_policy(Box::new(acl))
        .with_audit_logger(test_logger.clone())
        .build()
        .expect("Failed to build security middleware");

    // Test: Write operation
    let test_path = "/tmp/test_audit_write.txt";
    let result =
        write_file_with_middleware(test_path, b"test data".to_vec(), "alice", security).await;

    // Cleanup
    let _ = std::fs::remove_file(test_path);

    // Verify: Operation succeeded
    assert!(result.is_ok(), "Expected write to succeed");

    // Verify: Audit log captures the operation
    let events = test_logger.get_events();
    assert!(!events.is_empty(), "Expected audit event for file write");

    // Verify: User context is correct
    let has_alice = events.iter().any(|e| e.principal == "alice");
    assert!(has_alice, "Expected audit log for user 'alice'");

    // Verify: Decision was Allow
    let has_allow = events.iter().any(|e| e.decision.contains("Allow"));
    assert!(has_allow, "Expected Allow decision in audit log");
}

// ============================================================================
// Failed Operation Logging Tests
// ============================================================================

#[tokio::test]
async fn test_security_violation_logged_with_context() {
    // Setup: Create security middleware with test logger and deny policy
    let test_logger = Arc::new(TestSecurityAuditLogger::new());

    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "attacker".to_string(),
        "/secret/*".to_string(),
        vec!["*".to_string()],
        AclPolicy::Deny,
    ));

    let security = SecurityMiddlewareBuilder::new()
        .add_policy(Box::new(acl))
        .with_audit_logger(test_logger.clone())
        .build()
        .expect("Failed to build security middleware");

    // Test: Attempt to read denied file
    let result = read_file_with_middleware("/secret/passwords.txt", "attacker", security).await;

    // Verify: Operation was denied
    assert!(result.is_err(), "Expected operation to be denied");

    // Verify: Security violation was logged
    let events = test_logger.get_events();
    assert!(
        !events.is_empty(),
        "Expected audit event for security violation"
    );

    // Verify: Event contains attacker's identity
    let has_attacker = events.iter().any(|e| e.principal == "attacker");
    assert!(has_attacker, "Expected audit log for 'attacker' principal");

    // Verify: Event shows denial
    let has_deny = events.iter().any(|e| e.decision.contains("Deny"));
    assert!(
        has_deny,
        "Expected 'Deny' decision in audit log for security violation"
    );

    // Verify: Policy name is recorded
    let has_policy_info = events.iter().any(|e| !e.policy_applied.is_empty());
    assert!(
        has_policy_info,
        "Expected policy name to be recorded in audit log"
    );
}

#[tokio::test]
async fn test_permission_denied_logged_with_reason() {
    // Setup: RBAC policy that denies user without proper role
    let test_logger = Arc::new(TestSecurityAuditLogger::new());

    let permission = Permission::new(
        "file:write".to_string(),
        "Write File".to_string(),
        "Allows writing files".to_string(),
    );

    let role = Role::new("admin_role".to_string(), "Admin Role".to_string())
        .with_permission("file:write".to_string());

    let mut rbac = RoleBasedAccessControl::new();
    rbac = rbac
        .add_permission(permission)
        .add_role(role)
        .assign_roles("admin".to_string(), vec!["admin_role".to_string()]);
    // Note: "guest" has no roles

    let security = SecurityMiddlewareBuilder::new()
        .add_policy(Box::new(rbac))
        .with_audit_logger(test_logger.clone())
        .build()
        .expect("Failed to build security middleware");

    // Test: Guest tries to write (should be denied)
    let result =
        write_file_with_middleware("/tmp/test.txt", b"data".to_vec(), "guest", security).await;

    // Verify: Operation was denied
    assert!(result.is_err(), "Expected permission denied for guest");

    // Verify: Denial was logged
    let events = test_logger.get_events();
    assert!(!events.is_empty(), "Expected audit event for denial");

    // Verify: Guest user is in the log
    let has_guest = events.iter().any(|e| e.principal == "guest");
    assert!(has_guest, "Expected audit log for 'guest' user");

    // Verify: Denial reason is present
    let has_deny_reason = events.iter().any(|e| e.decision.contains("Deny"));
    assert!(
        has_deny_reason,
        "Expected denial reason in audit log, got: {:?}",
        events
    );
}

// ============================================================================
// Audit Event Verification Tests
// ============================================================================

#[tokio::test]
async fn test_audit_log_contains_session_id() {
    // Setup
    let test_logger = Arc::new(TestSecurityAuditLogger::new());

    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "user".to_string(),
        "*".to_string(),
        vec!["*".to_string()],
        AclPolicy::Allow,
    ));

    let security = SecurityMiddlewareBuilder::new()
        .add_policy(Box::new(acl))
        .with_audit_logger(test_logger.clone())
        .build()
        .expect("Failed to build security middleware");

    // Create test file
    let test_path = "/tmp/test_session.txt";
    std::fs::write(test_path, b"test").expect("Failed to create test file");

    // Test: Perform operation
    let _ = read_file_with_middleware(test_path, "user", security).await;

    // Cleanup
    let _ = std::fs::remove_file(test_path);

    // Verify: Session ID is present in audit log
    let events = test_logger.get_events();
    assert!(!events.is_empty(), "Expected audit events");

    let has_session_id = events.iter().any(|e| !e.session_id.is_empty());
    assert!(
        has_session_id,
        "Expected session_id to be present in audit log"
    );
}

#[tokio::test]
async fn test_multiple_operations_generate_separate_audit_logs() {
    // Setup
    let test_logger = Arc::new(TestSecurityAuditLogger::new());

    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "user".to_string(),
        "/tmp/*".to_string(),
        vec!["read".to_string(), "write".to_string()],
        AclPolicy::Allow,
    ));

    let security1 = SecurityMiddlewareBuilder::new()
        .add_policy(Box::new(acl.clone()))
        .with_audit_logger(test_logger.clone())
        .build()
        .expect("Failed to build security middleware");

    let security2 = SecurityMiddlewareBuilder::new()
        .add_policy(Box::new(acl))
        .with_audit_logger(test_logger.clone())
        .build()
        .expect("Failed to build security middleware");

    // Create test file
    let test_path1 = "/tmp/test_multi1.txt";
    let test_path2 = "/tmp/test_multi2.txt";
    std::fs::write(test_path1, b"test1").expect("Failed to create test file 1");
    std::fs::write(test_path2, b"test2").expect("Failed to create test file 2");

    // Test: Perform multiple operations
    let _ = read_file_with_middleware(test_path1, "user", security1).await;
    let _ = read_file_with_middleware(test_path2, "user", security2).await;

    // Cleanup
    let _ = std::fs::remove_file(test_path1);
    let _ = std::fs::remove_file(test_path2);

    // Verify: Multiple audit logs generated
    let event_count = test_logger.event_count();
    assert!(
        event_count >= 2,
        "Expected at least 2 audit events, got: {}",
        event_count
    );
}
