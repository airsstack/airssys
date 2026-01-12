//! Integration tests for airssys-osl security integration.
//!
//! Tests end-to-end permission checks through OslSecurityBridge
//! with realistic ACL scenarios.

use airssys_wasm::security::osl::OslSecurityBridge;
use airssys_osl::middleware::security::acl::{AccessControlList, AclEntry, AclPolicy};

#[test]
fn test_filesystem_access_control() {
    // Scenario: Component with filesystem read capability
    let mut acl = AccessControlList::new();

    // Grant read access to /app/data/*
    acl = acl.add_entry(AclEntry::new(
        "component-fs-001".to_string(),
        "/app/data/*".to_string(),
        vec!["read".to_string()],
        AclPolicy::Allow,
    ));

    let bridge = OslSecurityBridge::new(acl);

    // Test: Allowed paths
    assert!(bridge
        .check_permission("component-fs-001", "/app/data/config.txt", "read")
        .is_ok());
    assert!(bridge
        .check_permission("component-fs-001", "/app/data/subdir/file.txt", "read")
        .is_ok());

    // Test: Denied operations
    assert!(bridge
        .check_permission("component-fs-001", "/app/data/secret.txt", "write")
        .is_err());
    assert!(bridge
        .check_permission("component-fs-001", "/app/config/settings.txt", "read")
        .is_err());
}

#[test]
fn test_network_access_control() {
    // Scenario: Component with network connect capability
    let mut acl = AccessControlList::new();

    acl = acl.add_entry(AclEntry::new(
        "component-net-002".to_string(),
        "api.example.com:443".to_string(),
        vec!["connect".to_string()],
        AclPolicy::Allow,
    ));

    let bridge = OslSecurityBridge::new(acl);

    // Test: Allowed connections
    assert!(bridge
        .check_permission("component-net-002", "api.example.com:443", "connect")
        .is_ok());

    // Test: Denied connections
    assert!(bridge
        .check_permission("component-net-002", "other-api.com:443", "connect")
        .is_err());
    assert!(bridge
        .check_permission("component-net-002", "api.example.com:80", "connect")
        .is_err());
}

#[test]
fn test_component_isolation() {
    // Scenario: Multiple components with different access levels
    let mut acl = AccessControlList::new();

    // Component A: Can read /app/public/*
    acl = acl.add_entry(AclEntry::new(
        "component-a".to_string(),
        "/app/public/*".to_string(),
        vec!["read".to_string()],
        AclPolicy::Allow,
    ));

    // Component B: Can read/write /app/private/*
    acl = acl.add_entry(AclEntry::new(
        "component-b".to_string(),
        "/app/private/*".to_string(),
        vec!["read".to_string(), "write".to_string()],
        AclPolicy::Allow,
    ));

    let bridge = OslSecurityBridge::new(acl);

    // Test: Component A access
    assert!(bridge
        .check_permission("component-a", "/app/public/file.txt", "read")
        .is_ok());
    assert!(bridge
        .check_permission("component-a", "/app/private/secret.txt", "read")
        .is_err());

    // Test: Component B access
    assert!(bridge
        .check_permission("component-b", "/app/private/data.txt", "read")
        .is_ok());
    assert!(bridge
        .check_permission("component-b", "/app/private/data.txt", "write")
        .is_ok());
    assert!(bridge
        .check_permission("component-b", "/app/public/file.txt", "read")
        .is_err());
}

#[test]
fn test_deny_by_default_behavior() {
    // Scenario: New component with no ACL entries
    let acl = AccessControlList::new();
    let bridge = OslSecurityBridge::new(acl);

    // All operations should be denied
    assert!(bridge
        .check_permission("new-component", "/any/resource", "any-action")
        .is_err());
    assert!(bridge
        .check_permission("new-component", "/app/data", "read")
        .is_err());
    assert!(bridge
        .check_permission("new-component", "localhost:8080", "connect")
        .is_err());
}

#[test]
fn test_pattern_matching_glob_patterns() {
    // Scenario: Component with wildcard patterns
    let mut acl = AccessControlList::new();

    // Grant access to all logs
    acl = acl.add_entry(AclEntry::new(
        "component-logger".to_string(),
        "/var/log/**/*.log".to_string(),
        vec!["read".to_string()],
        AclPolicy::Allow,
    ));

    let bridge = OslSecurityBridge::new(acl);

    // Test: Pattern matching
    assert!(bridge
        .check_permission("component-logger", "/var/log/app.log", "read")
        .is_ok());
    assert!(bridge
        .check_permission("component-logger", "/var/log/subdir/error.log", "read")
        .is_ok());
    assert!(bridge
        .check_permission("component-logger", "/var/log/subdir/nested/debug.log", "read")
        .is_ok());

    // Test: Non-matching paths
    assert!(bridge
        .check_permission("component-logger", "/var/log/config.txt", "read")
        .is_err());
    assert!(bridge
        .check_permission("component-logger", "/var/data/file.log", "read")
        .is_err());
}

#[test]
fn test_multiple_permissions() {
    // Scenario: Component with multiple permissions on same resource
    let mut acl = AccessControlList::new();

    acl = acl.add_entry(AclEntry::new(
        "component-multi".to_string(),
        "/app/shared/*".to_string(),
        vec!["read".to_string(), "write".to_string(), "delete".to_string()],
        AclPolicy::Allow,
    ));

    let bridge = OslSecurityBridge::new(acl);

    // Test: All allowed permissions
    assert!(bridge
        .check_permission("component-multi", "/app/shared/data.txt", "read")
        .is_ok());
    assert!(bridge
        .check_permission("component-multi", "/app/shared/data.txt", "write")
        .is_ok());
    assert!(bridge
        .check_permission("component-multi", "/app/shared/data.txt", "delete")
        .is_ok());

    // Test: Other permissions denied
    assert!(bridge
        .check_permission("component-multi", "/app/shared/data.txt", "execute")
        .is_err());
}

#[test]
fn test_security_context_attributes() {
    // Scenario: Verify context attributes are passed correctly
    let mut acl = AccessControlList::new();

    acl = acl.add_entry(AclEntry::new(
        "component-ctx".to_string(),
        "/resource".to_string(),
        vec!["action".to_string()],
        AclPolicy::Allow,
    ));

    let bridge = OslSecurityBridge::new(acl);

    // Verify context is built and used internally by check_permission
    let decision = bridge.check_permission("component-ctx", "/resource", "action");
    assert!(decision.is_ok());
}
