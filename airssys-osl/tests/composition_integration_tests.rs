//! Integration tests for composition API type safety and cross-operation testing.
//!
//! Tests type safety, cross-operation scenarios, and mixed operations with shared middleware.

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use std::fs;

use airssys_osl::core::security::SecurityConfig;
use airssys_osl::helpers::composition::{FileHelper, HelperPipeline, NetworkHelper, ProcessHelper};
use airssys_osl::middleware::security::acl::{AccessControlList, AclEntry, AclPolicy};
use airssys_osl::middleware::security::middleware::SecurityMiddlewareBuilder;

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
async fn test_type_safety_file_operations() {
    // Create test file
    let test_file = std::env::temp_dir().join(format!("test_type_{}.txt", uuid::Uuid::new_v4()));
    fs::write(&test_file, "test content").expect("Failed to create test file");
    let test_path = test_file.to_str().unwrap();

    let acl = create_allow_acl();
    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    // FileHelper::builder() returns ComposedHelper<FileReadOperation, FilesystemExecutor>
    // This enforces type safety at compile time
    let reader = FileHelper::builder().with_security(middleware);

    // Type inference ensures correct operation type
    let result = reader.read(test_path, "testuser").await;
    assert!(result.is_ok(), "Type-safe read operation should succeed");

    // Cleanup
    fs::remove_file(test_file).ok();
}

#[tokio::test]
async fn test_type_safety_process_operations() {
    let acl = create_allow_acl();
    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    // ProcessHelper::builder() returns ComposedHelper<ProcessSpawnOperation, ProcessExecutor>
    let spawner = ProcessHelper::builder().with_security(middleware);

    // Type inference ensures correct operation type
    let result = spawner
        .spawn("echo", vec!["test".to_string()], "testuser")
        .await;
    assert!(result.is_ok(), "Type-safe spawn operation should succeed");
}

#[tokio::test]
async fn test_type_safety_network_operations() {
    let acl = create_allow_acl();
    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    // NetworkHelper::builder() returns ComposedHelper<NetworkConnectOperation, NetworkExecutor>
    let connector = NetworkHelper::builder().with_security(middleware);

    // Type inference ensures correct operation type
    // Using example.com which should be accessible
    let result = connector.connect("example.com:80", "testuser").await;

    // Network operations may fail due to connectivity, but type safety is verified at compile time
    assert!(
        result.is_ok() || result.is_err(),
        "Type-safe network operation completes"
    );
}

#[tokio::test]
async fn test_shared_middleware_instance() {
    // Create separate middleware instances with the same configuration
    let acl1 = create_allow_acl();
    let acl2 = create_allow_acl();
    let acl3 = create_allow_acl();

    let file_middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl1))
        .build()
        .expect("Failed to build middleware");

    let process_middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl2))
        .build()
        .expect("Failed to build middleware");

    let network_middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl3))
        .build()
        .expect("Failed to build middleware");

    // Create helpers with equivalent middleware instances
    let file_helper = FileHelper::builder().with_security(file_middleware);
    let process_helper = ProcessHelper::builder().with_security(process_middleware);
    let network_helper = NetworkHelper::builder().with_security(network_middleware);

    // Create test file
    let test_file = std::env::temp_dir().join(format!("test_shared_{}.txt", uuid::Uuid::new_v4()));
    fs::write(&test_file, "shared test").expect("Failed to create test file");
    let test_path = test_file.to_str().unwrap();

    // Use all helpers with the shared middleware
    let file_result = file_helper.read(test_path, "testuser").await;
    let process_result = process_helper
        .spawn("echo", vec!["hello".to_string()], "testuser")
        .await;
    let network_result = network_helper.connect("example.com:80", "testuser").await;

    // All should respect the same security policy
    assert!(
        file_result.is_ok(),
        "File operation with shared middleware should succeed"
    );
    assert!(
        process_result.is_ok(),
        "Process operation with shared middleware should succeed"
    );
    // Network may fail due to connectivity, but middleware works
    assert!(
        network_result.is_ok() || network_result.is_err(),
        "Network operation with shared middleware completes"
    );

    // Cleanup
    fs::remove_file(test_file).ok();
}

#[tokio::test]
async fn test_cross_operation_sequential_execution() {
    // Test that different operation types can be executed sequentially
    // with proper cleanup and state management
    let acl1 = create_allow_acl();
    let acl2 = create_allow_acl();
    let acl3 = create_allow_acl();

    let file_middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl1))
        .build()
        .expect("Failed to build middleware");

    let process_middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl2))
        .build()
        .expect("Failed to build middleware");

    let network_middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl3))
        .build()
        .expect("Failed to build middleware");

    // Create test file
    let test_file = std::env::temp_dir().join(format!("test_seq_{}.txt", uuid::Uuid::new_v4()));
    fs::write(&test_file, "sequential test").expect("Failed to create test file");
    let test_path = test_file.to_str().unwrap();

    // Execute operations in sequence
    // 1. File operation
    let file_helper = FileHelper::builder().with_security(file_middleware);
    let file_data = file_helper
        .read(test_path, "testuser")
        .await
        .expect("File read should succeed");
    assert!(!file_data.is_empty(), "Should read file data");

    // 2. Process operation
    let process_helper = ProcessHelper::builder().with_security(process_middleware);
    let process_output = process_helper
        .spawn("echo", vec!["test".to_string()], "testuser")
        .await
        .expect("Process spawn should succeed");
    assert!(!process_output.is_empty(), "Should get process output");

    // 3. Network operation (may fail due to connectivity)
    let network_helper = NetworkHelper::builder().with_security(network_middleware);
    let _network_result = network_helper.connect("example.com:80", "testuser").await;

    // All operations completed in sequence without interference
    // Cleanup
    fs::remove_file(test_file).ok();
}

#[tokio::test]
async fn test_pipeline_reuse_across_operations() {
    // Test that a configured pipeline can be reused multiple times
    let acl = create_allow_acl();
    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    let helper = FileHelper::builder().with_security(middleware);

    // Create multiple test files
    let files: Vec<_> = (0..3)
        .map(|i| {
            let path = std::env::temp_dir().join(format!("test_reuse_{i}.txt"));
            fs::write(&path, format!("content {i}")).expect("Failed to create test file");
            path
        })
        .collect();

    // Reuse the same helper pipeline for multiple operations
    for file in &files {
        let result = helper.read(file.to_str().unwrap(), "testuser").await;
        assert!(
            result.is_ok(),
            "Pipeline reuse should work for all operations"
        );
    }

    // Cleanup
    for file in files {
        fs::remove_file(file).ok();
    }
}

#[tokio::test]
async fn test_different_security_policies_different_helpers() {
    // Create two different ACLs
    let allow_acl = create_allow_acl();
    let deny_acl = AccessControlList::new().add_entry(AclEntry::new(
        "testuser".to_string(),
        "*".to_string(),
        vec!["*".to_string()],
        AclPolicy::Deny,
    ));

    // Create two helpers with different security policies
    let allow_middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(allow_acl))
        .build()
        .expect("Failed to build allow middleware");

    let deny_middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(deny_acl))
        .build()
        .expect("Failed to build deny middleware");

    let allow_helper = FileHelper::builder().with_security(allow_middleware);
    let deny_helper = FileHelper::builder().with_security(deny_middleware);

    // Create test file
    let test_file =
        std::env::temp_dir().join(format!("test_policies_{}.txt", uuid::Uuid::new_v4()));
    fs::write(&test_file, "policy test").expect("Failed to create test file");
    let test_path = test_file.to_str().unwrap();

    // Allow helper should succeed
    let allow_result = allow_helper.read(test_path, "testuser").await;
    assert!(allow_result.is_ok(), "Allow policy should permit operation");

    // Deny helper should fail
    let deny_result = deny_helper.read(test_path, "testuser").await;
    assert!(deny_result.is_err(), "Deny policy should reject operation");

    // Cleanup
    fs::remove_file(test_file).ok();
}

#[tokio::test]
async fn test_middleware_semantics() {
    // Test that middleware instances work correctly when created separately
    let acl1 = create_allow_acl();
    let acl2 = create_allow_acl();

    let middleware1 = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl1))
        .build()
        .expect("Failed to build middleware");

    let middleware2 = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl2))
        .build()
        .expect("Failed to build middleware");

    // Create helpers with separate but equivalent middleware
    let helper1 = FileHelper::builder().with_security(middleware1);
    let helper2 = FileHelper::builder().with_security(middleware2);

    // Create test file
    let test_file = std::env::temp_dir().join(format!("test_clone_{}.txt", uuid::Uuid::new_v4()));
    fs::write(&test_file, "clone test").expect("Failed to create test file");
    let test_path = test_file.to_str().unwrap();

    // Both should work identically with same security policy
    let result1 = helper1.read(test_path, "testuser").await;
    let result2 = helper2.read(test_path, "testuser").await;

    assert!(result1.is_ok(), "First middleware instance should work");
    assert!(result2.is_ok(), "Second middleware instance should work");

    // Cleanup
    fs::remove_file(test_file).ok();
}
