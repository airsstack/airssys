//! Integration Tests for Capability Check API (Task 3.1)
//!
//! These tests verify the complete capability checking workflow:
//! - Component registration and unregistration
//! - Capability checks (granted/denied scenarios)
//! - airssys-osl ACL integration
//! - Thread safety and concurrency
//! - Error handling and edge cases
//!
//! # Test Organization
//!
//! - **Basic Functionality**: Registration, unregistration, basic checks
//! - **Access Control**: Pattern matching, permission checks, deny-by-default
//! - **Integration**: airssys-osl ACL evaluation, context conversion
//! - **Concurrency**: Thread safety, concurrent access patterns
//! - **Edge Cases**: Empty capabilities, unregistered components, invalid patterns

// Allow test-specific patterns
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
use airssys_wasm::security::enforcement::{
    check_capability, register_component, unregister_component, CapabilityCheckError,
    CapabilityCheckResult, CapabilityChecker,
};
use airssys_wasm::security::{WasmCapability, WasmCapabilitySet, WasmSecurityContext};

// ============================================================================
// Basic Functionality Tests
// ============================================================================

/// Test: Component registration and check with filesystem capability.
///
/// Verifies that a component with filesystem read capability can access
/// files matching the declared pattern.
#[test]
fn test_basic_filesystem_capability_granted() {
    let checker = CapabilityChecker::new();

    let capabilities = WasmCapabilitySet::new().grant(WasmCapability::Filesystem {
        paths: vec!["/app/data/*".to_string()],
        permissions: vec!["read".to_string()],
    });

    let security_ctx = WasmSecurityContext::new("test-fs-granted".to_string(), capabilities);
    checker
        .register_component(security_ctx)
        .expect("registration failed");

    // Check access to file matching pattern
    let result = checker.check("test-fs-granted", "/app/data/config.json", "read");
    assert!(
        result.is_granted(),
        "Expected access granted, got: {:?}",
        result
    );
}

/// Test: Component registration and check with network capability.
///
/// Verifies that a component with network connect capability can access
/// endpoints matching the declared pattern.
#[test]
fn test_basic_network_capability_granted() {
    let checker = CapabilityChecker::new();

    let capabilities = WasmCapabilitySet::new().grant(WasmCapability::Network {
        endpoints: vec!["api.example.com:443".to_string()],
        permissions: vec!["connect".to_string()],
    });

    let security_ctx = WasmSecurityContext::new("test-net-granted".to_string(), capabilities);
    checker
        .register_component(security_ctx)
        .expect("registration failed");

    // Check access to declared endpoint
    let result = checker.check("test-net-granted", "api.example.com:443", "connect");
    assert!(
        result.is_granted(),
        "Expected access granted, got: {:?}",
        result
    );
}

/// Test: Component registration and check with storage capability.
///
/// Verifies that a component with storage capability can access
/// namespaces matching the declared pattern.
#[test]
fn test_basic_storage_capability_granted() {
    let checker = CapabilityChecker::new();

    let capabilities = WasmCapabilitySet::new().grant(WasmCapability::Storage {
        namespaces: vec!["component:test-id:data:*".to_string()],
        permissions: vec!["read".to_string(), "write".to_string()],
    });

    let security_ctx = WasmSecurityContext::new("test-storage-granted".to_string(), capabilities);
    checker
        .register_component(security_ctx)
        .expect("registration failed");

    // Check read access
    let result = checker.check(
        "test-storage-granted",
        "component:test-id:data:config",
        "read",
    );
    assert!(
        result.is_granted(),
        "Expected read access granted, got: {:?}",
        result
    );

    // Check write access
    let result = checker.check(
        "test-storage-granted",
        "component:test-id:data:cache",
        "write",
    );
    assert!(
        result.is_granted(),
        "Expected write access granted, got: {:?}",
        result
    );
}

/// Test: Component with no capabilities is denied access.
///
/// Verifies deny-by-default: components without capabilities cannot access anything.
#[test]
fn test_no_capabilities_denied() {
    let checker = CapabilityChecker::new();

    // Register component with empty capability set
    let security_ctx = WasmSecurityContext::new(
        "test-no-caps".to_string(),
        WasmCapabilitySet::new(), // Empty
    );
    checker
        .register_component(security_ctx)
        .expect("registration failed");

    // Any access attempt should be denied
    let result = checker.check("test-no-caps", "/app/data/file.json", "read");
    assert!(result.is_denied(), "Expected access denied, got: {:?}", result);
    assert!(result
        .denial_reason()
        .unwrap()
        .contains("no capabilities declared"));
}

/// Test: Unregistered component is denied access.
///
/// Verifies that components must be registered before capability checks.
#[test]
fn test_unregistered_component_denied() {
    let checker = CapabilityChecker::new();

    let result = checker.check("unregistered-component", "/app/data/file.json", "read");
    assert!(result.is_denied(), "Expected access denied, got: {:?}", result);
    assert!(result
        .denial_reason()
        .unwrap()
        .contains("not registered"));
}

/// Test: Component unregistration.
///
/// Verifies that components can be unregistered and subsequent checks fail.
#[test]
fn test_component_unregistration() {
    let checker = CapabilityChecker::new();

    let capabilities = WasmCapabilitySet::new().grant(WasmCapability::Filesystem {
        paths: vec!["/app/data/*".to_string()],
        permissions: vec!["read".to_string()],
    });

    let security_ctx = WasmSecurityContext::new("test-unreg".to_string(), capabilities);
    checker
        .register_component(security_ctx)
        .expect("registration failed");

    // Verify access before unregistration
    let result = checker.check("test-unreg", "/app/data/file.json", "read");
    assert!(result.is_granted());

    // Unregister component
    checker
        .unregister_component("test-unreg")
        .expect("unregistration failed");

    // Verify access denied after unregistration
    let result = checker.check("test-unreg", "/app/data/file.json", "read");
    assert!(result.is_denied());
    assert!(result.denial_reason().unwrap().contains("not registered"));
}

// ============================================================================
// Access Control Tests
// ============================================================================

/// Test: Pattern mismatch - component cannot access resources outside declared pattern.
///
/// Verifies that capability pattern matching enforces boundaries.
#[test]
fn test_pattern_mismatch_denied() {
    let checker = CapabilityChecker::new();

    let capabilities = WasmCapabilitySet::new().grant(WasmCapability::Filesystem {
        paths: vec!["/app/data/*".to_string()], // Only /app/data/*
        permissions: vec!["read".to_string()],
    });

    let security_ctx = WasmSecurityContext::new("test-pattern-mismatch".to_string(), capabilities);
    checker
        .register_component(security_ctx)
        .expect("registration failed");

    // Attempt to access resource outside pattern
    let result = checker.check("test-pattern-mismatch", "/etc/passwd", "read");
    assert!(result.is_denied(), "Expected access denied, got: {:?}", result);
}

/// Test: Permission mismatch - component cannot use undeclared permissions.
///
/// Verifies that permission checks enforce declared permissions.
#[test]
fn test_permission_mismatch_denied() {
    let checker = CapabilityChecker::new();

    let capabilities = WasmCapabilitySet::new().grant(WasmCapability::Filesystem {
        paths: vec!["/app/data/*".to_string()],
        permissions: vec!["read".to_string()], // Only read, not write
    });

    let security_ctx = WasmSecurityContext::new("test-perm-mismatch".to_string(), capabilities);
    checker
        .register_component(security_ctx)
        .expect("registration failed");

    // Read access should be granted
    let result = checker.check("test-perm-mismatch", "/app/data/file.json", "read");
    assert!(result.is_granted());

    // Write access should be denied
    let result = checker.check("test-perm-mismatch", "/app/data/file.json", "write");
    assert!(result.is_denied(), "Expected access denied, got: {:?}", result);
}

/// Test: Glob pattern matching - wildcard in path.
///
/// Verifies that glob patterns work correctly for path matching.
#[test]
fn test_glob_pattern_matching() {
    let checker = CapabilityChecker::new();

    let capabilities = WasmCapabilitySet::new().grant(WasmCapability::Filesystem {
        paths: vec!["/app/data/*.json".to_string()], // Only .json files
        permissions: vec!["read".to_string()],
    });

    let security_ctx = WasmSecurityContext::new("test-glob".to_string(), capabilities);
    checker
        .register_component(security_ctx)
        .expect("registration failed");

    // .json file should be granted
    let result = checker.check("test-glob", "/app/data/config.json", "read");
    assert!(
        result.is_granted(),
        "Expected access granted for .json file, got: {:?}",
        result
    );

    // .txt file should be denied
    let result = checker.check("test-glob", "/app/data/readme.txt", "read");
    assert!(
        result.is_denied(),
        "Expected access denied for .txt file, got: {:?}",
        result
    );
}

/// Test: Multiple capabilities - component has access to multiple resources.
///
/// Verifies that components with multiple capabilities can access all declared resources.
#[test]
fn test_multiple_capabilities_granted() {
    let checker = CapabilityChecker::new();

    let capabilities = WasmCapabilitySet::new()
        .grant(WasmCapability::Filesystem {
            paths: vec!["/app/config/*".to_string(), "/app/data/*".to_string()],
            permissions: vec!["read".to_string()],
        })
        .grant(WasmCapability::Network {
            endpoints: vec!["api.example.com:443".to_string()],
            permissions: vec!["connect".to_string()],
        });

    let security_ctx = WasmSecurityContext::new("test-multi-caps".to_string(), capabilities);
    checker
        .register_component(security_ctx)
        .expect("registration failed");

    // All declared resources should be accessible
    assert!(checker
        .check("test-multi-caps", "/app/config/settings.toml", "read")
        .is_granted());
    assert!(checker
        .check("test-multi-caps", "/app/data/file.json", "read")
        .is_granted());
    assert!(checker
        .check("test-multi-caps", "api.example.com:443", "connect")
        .is_granted());

    // Undeclared resource should be denied
    assert!(checker
        .check("test-multi-caps", "/etc/passwd", "read")
        .is_denied());
}

/// Test: Multiple permissions - component has read and write access.
///
/// Verifies that components can have multiple permissions for the same resource.
#[test]
fn test_multiple_permissions_granted() {
    let checker = CapabilityChecker::new();

    let capabilities = WasmCapabilitySet::new().grant(WasmCapability::Filesystem {
        paths: vec!["/app/data/*".to_string()],
        permissions: vec!["read".to_string(), "write".to_string()],
    });

    let security_ctx = WasmSecurityContext::new("test-multi-perms".to_string(), capabilities);
    checker
        .register_component(security_ctx)
        .expect("registration failed");

    // Both read and write should be granted
    assert!(checker
        .check("test-multi-perms", "/app/data/file.json", "read")
        .is_granted());
    assert!(checker
        .check("test-multi-perms", "/app/data/file.json", "write")
        .is_granted());

    // Execute permission (not declared) should be denied
    assert!(checker
        .check("test-multi-perms", "/app/data/script.sh", "execute")
        .is_denied());
}

// ============================================================================
// Integration Tests (airssys-osl ACL)
// ============================================================================

/// Test: ACL pattern matching integration.
///
/// Verifies that airssys-osl's ACL pattern matching works correctly
/// with WASM capability declarations.
#[test]
fn test_acl_integration_pattern_matching() {
    let checker = CapabilityChecker::new();

    let capabilities = WasmCapabilitySet::new().grant(WasmCapability::Filesystem {
        paths: vec!["/app/data/**/*.log".to_string()], // Recursive glob
        permissions: vec!["read".to_string()],
    });

    let security_ctx = WasmSecurityContext::new("test-acl-pattern".to_string(), capabilities);
    checker
        .register_component(security_ctx)
        .expect("registration failed");

    // Nested .log files should be accessible
    assert!(checker
        .check("test-acl-pattern", "/app/data/logs/app.log", "read")
        .is_granted());
    assert!(checker
        .check("test-acl-pattern", "/app/data/subdir/debug.log", "read")
        .is_granted());

    // Non-.log files should be denied
    assert!(checker
        .check("test-acl-pattern", "/app/data/logs/config.json", "read")
        .is_denied());
}

/// Test: Multiple ACL entries from single capability.
///
/// Verifies that capabilities with multiple patterns create multiple ACL entries.
#[test]
fn test_acl_multiple_entries() {
    let checker = CapabilityChecker::new();

    let capabilities = WasmCapabilitySet::new().grant(WasmCapability::Filesystem {
        paths: vec![
            "/app/config/*".to_string(),
            "/app/data/*".to_string(),
            "/app/logs/*".to_string(),
        ],
        permissions: vec!["read".to_string()],
    });

    let security_ctx = WasmSecurityContext::new("test-acl-multi".to_string(), capabilities);
    checker
        .register_component(security_ctx)
        .expect("registration failed");

    // All three directories should be accessible
    assert!(checker
        .check("test-acl-multi", "/app/config/settings.toml", "read")
        .is_granted());
    assert!(checker
        .check("test-acl-multi", "/app/data/db.sqlite", "read")
        .is_granted());
    assert!(checker
        .check("test-acl-multi", "/app/logs/app.log", "read")
        .is_granted());
}

// ============================================================================
// Concurrency Tests
// ============================================================================

/// Test: Concurrent component registration.
///
/// Verifies that multiple threads can register components concurrently
/// without data races or deadlocks.
#[test]
fn test_concurrent_registration() {
    use std::sync::Arc;
    use std::thread;

    let checker = Arc::new(CapabilityChecker::new());
    let mut handles = vec![];

    // Spawn 10 threads that register components
    for i in 0..10 {
        let checker_clone = Arc::clone(&checker);
        let handle = thread::spawn(move || {
            let component_id = format!("concurrent-reg-{}", i);
            let capabilities = WasmCapabilitySet::new().grant(WasmCapability::Filesystem {
                paths: vec![format!("/app/data-{}/*", i)],  // Fixed: Add wildcard
                permissions: vec!["read".to_string()],
            });

            let security_ctx = WasmSecurityContext::new(component_id, capabilities);
            checker_clone
                .register_component(security_ctx)
                .expect("concurrent registration failed");
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("thread panicked");
    }

    // All 10 components should be registered
    assert_eq!(checker.component_count(), 10);
}

/// Test: Concurrent capability checking.
///
/// Verifies that multiple threads can check capabilities concurrently
/// without data races or deadlocks.
#[test]
fn test_concurrent_checking() {
    use std::sync::Arc;
    use std::thread;

    let checker = Arc::new(CapabilityChecker::new());

    // Register a single component
    let capabilities = WasmCapabilitySet::new().grant(WasmCapability::Filesystem {
        paths: vec!["/app/data/*".to_string()],
        permissions: vec!["read".to_string()],
    });
    let security_ctx = WasmSecurityContext::new("concurrent-check".to_string(), capabilities);
    checker
        .register_component(security_ctx)
        .expect("registration failed");

    let mut handles = vec![];

    // Spawn 20 threads that check capabilities concurrently
    for i in 0..20 {
        let checker_clone = Arc::clone(&checker);
        let handle = thread::spawn(move || {
            let resource = format!("/app/data/file-{}.json", i);
            let result = checker_clone.check("concurrent-check", &resource, "read");
            assert!(
                result.is_granted(),
                "Concurrent check failed for {}",
                resource
            );
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("thread panicked");
    }
}

/// Test: Concurrent registration and checking.
///
/// Verifies that threads can register and check capabilities concurrently
/// without interfering with each other.
#[test]
fn test_concurrent_mixed_operations() {
    use std::sync::Arc;
    use std::thread;

    let checker = Arc::new(CapabilityChecker::new());
    let mut handles = vec![];

    // Spawn threads that register and immediately check
    for i in 0..10 {
        let checker_clone = Arc::clone(&checker);
        let handle = thread::spawn(move || {
            let component_id = format!("concurrent-mixed-{}", i);
            let resource = format!("/app/data-{}/file.json", i);
            let capabilities = WasmCapabilitySet::new().grant(WasmCapability::Filesystem {
                paths: vec![format!("/app/data-{}/*", i)],  // Fixed: Add wildcard
                permissions: vec!["read".to_string()],
            });

            // Register
            let security_ctx = WasmSecurityContext::new(component_id.clone(), capabilities);
            checker_clone
                .register_component(security_ctx)
                .expect("registration failed");

            // Check immediately after registration
            let result = checker_clone.check(&component_id, &resource, "read");
            assert!(result.is_granted(), "Check failed after registration");
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("thread panicked");
    }

    assert_eq!(checker.component_count(), 10);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

/// Test: Duplicate registration error.
///
/// Verifies that attempting to register the same component twice fails.
#[test]
fn test_duplicate_registration_error() {
    let checker = CapabilityChecker::new();
    let capabilities = WasmCapabilitySet::new();

    let security_ctx1 = WasmSecurityContext::new("test-dup-reg".to_string(), capabilities.clone());
    let security_ctx2 = WasmSecurityContext::new("test-dup-reg".to_string(), capabilities);

    // First registration should succeed
    checker
        .register_component(security_ctx1)
        .expect("first registration failed");

    // Second registration should fail
    let result = checker.register_component(security_ctx2);
    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(CapabilityCheckError::ComponentAlreadyRegistered { .. })
    ));
}

/// Test: Unregister non-existent component error.
///
/// Verifies that attempting to unregister a non-existent component fails.
#[test]
fn test_unregister_non_existent_error() {
    let checker = CapabilityChecker::new();
    let result = checker.unregister_component("non-existent");
    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(CapabilityCheckError::ComponentNotFound { .. })
    ));
}

/// Test: CapabilityCheckResult::to_result() conversion.
///
/// Verifies that check results can be converted to Result types for error handling.
#[test]
fn test_result_conversion() {
    let granted = CapabilityCheckResult::Granted;
    assert!(granted.to_result().is_ok());

    let denied = CapabilityCheckResult::Denied("Access denied".to_string());
    let result = denied.to_result();
    assert!(result.is_err());
    assert!(matches!(result, Err(CapabilityCheckError::AccessDenied { .. })));
}

// ============================================================================
// Global API Tests
// ============================================================================

/// Test: Global register_component() function.
///
/// Verifies that the global convenience API works correctly.
#[test]
fn test_global_register_component() {
    let component_id = format!("global-test-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos());
    
    let capabilities = WasmCapabilitySet::new().grant(WasmCapability::Filesystem {
        paths: vec!["/app/data/*".to_string()],
        permissions: vec!["read".to_string()],
    });

    let security_ctx = WasmSecurityContext::new(component_id.clone(), capabilities);
    let result = register_component(security_ctx);
    assert!(result.is_ok());

    // Cleanup
    let _ = unregister_component(&component_id);
}

/// Test: Global check_capability() function.
///
/// Verifies that the global check function works correctly.
#[test]
fn test_global_check_capability() {
    let component_id = format!("global-check-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos());

    let capabilities = WasmCapabilitySet::new().grant(WasmCapability::Filesystem {
        paths: vec!["/app/data/*".to_string()],
        permissions: vec!["read".to_string()],
    });

    let security_ctx = WasmSecurityContext::new(component_id.clone(), capabilities);
    register_component(security_ctx).expect("registration failed");

    // Check capability
    let result = check_capability(&component_id, "/app/data/file.json", "read");
    assert!(result.is_ok());

    // Cleanup
    let _ = unregister_component(&component_id);
}

/// Test: Global unregister_component() function.
///
/// Verifies that the global unregister function works correctly.
#[test]
fn test_global_unregister_component() {
    let component_id = format!("global-unreg-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos());

    let security_ctx = WasmSecurityContext::new(component_id.clone(), WasmCapabilitySet::new());
    register_component(security_ctx).expect("registration failed");

    let result = unregister_component(&component_id);
    assert!(result.is_ok());

    // Verify component is no longer registered
    let check_result = check_capability(&component_id, "/app/data/file.json", "read");
    assert!(check_result.is_err());
}
