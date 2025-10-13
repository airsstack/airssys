//! Basic Composition API Example
//!
//! This example demonstrates the fundamental usage of the Level 3 trait-based composition API.
//! It shows how to:
//! - Create helpers with security middleware
//! - Perform file operations with security enforcement
//! - Execute process operations with access control
//! - Perform network operations with security policies
//!
//! Run with: `cargo run --example composition_basic`

// Allow expect/unwrap in examples - this is demonstration code
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::PathBuf;

use airssys_osl::core::security::SecurityConfig;
use airssys_osl::helpers::composition::{FileHelper, HelperPipeline, NetworkHelper, ProcessHelper};
use airssys_osl::middleware::security::acl::{AccessControlList, AclEntry, AclPolicy};
use airssys_osl::middleware::security::middleware::SecurityMiddlewareBuilder;

/// Create a temporary test file for demonstration
fn create_demo_file() -> PathBuf {
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("composition_demo.txt");
    fs::write(
        &file_path,
        "This is a demonstration file for the AirsSys OSL composition API.\n\
         It shows how security policies are enforced during file operations.",
    )
    .expect("Failed to create demo file");
    file_path
}

/// Create a permissive ACL that allows all operations for demo purposes
fn create_permissive_acl() -> AccessControlList {
    AccessControlList::new().add_entry(AclEntry::new(
        "demo_user".to_string(),
        "*".to_string(),       // All resources
        vec!["*".to_string()], // All permissions
        AclPolicy::Allow,
    ))
}

/// Create a restrictive ACL that allows limited operations
/// Note: Since the composition helpers don't currently set ACL resource attributes,
/// we use a wildcard pattern for demonstration purposes
fn create_restrictive_acl() -> AccessControlList {
    AccessControlList::new().add_entry(AclEntry::new(
        "demo_user".to_string(),
        "*".to_string(),                               // All resources (wildcard)
        vec!["read".to_string(), "spawn".to_string()], // Limited permissions
        AclPolicy::Allow,
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AirsSys OSL Composition API - Basic Usage Example ===\n");

    // ========================================================================
    // Part 1: File Operations with Permissive Security
    // ========================================================================

    println!("Part 1: File Operations with Permissive Security");
    println!("------------------------------------------------");

    // Create a test file
    let demo_file = create_demo_file();
    println!("Created demo file: {}", demo_file.display());

    // Create security middleware with permissive ACL
    let permissive_acl = create_permissive_acl();
    let permissive_middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(permissive_acl))
        .build()?;

    // Create FileHelper with security
    let file_helper = FileHelper::builder().with_security(permissive_middleware);

    // Perform file read operation
    println!("\n✓ Reading file with permissive policy...");
    let content = file_helper
        .read(demo_file.to_str().unwrap(), "demo_user")
        .await?;

    println!("  Read {} bytes:", content.len());
    println!("  Content: {}", String::from_utf8_lossy(&content));

    // ========================================================================
    // Part 2: File Operations with Restrictive Security
    // ========================================================================

    println!("\n\nPart 2: File Operations with Restrictive Security");
    println!("--------------------------------------------------");

    // Create security middleware with restrictive ACL
    let restrictive_acl = create_restrictive_acl();
    let restrictive_middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(restrictive_acl))
        .build()?;

    // Create FileHelper with restrictive security
    let restricted_file_helper = FileHelper::builder().with_security(restrictive_middleware);

    // Read operation should succeed (allowed by policy)
    println!("\n✓ Reading file with restrictive policy...");
    let content = restricted_file_helper
        .read(demo_file.to_str().unwrap(), "demo_user")
        .await?;
    println!("  Successfully read {} bytes", content.len());

    // Note: Write operations would be denied by the restrictive policy
    // (commented out as FileHelper::builder() currently only supports read operations)
    println!("\n  Note: Write operations would be denied by restrictive policy");
    println!("  (FileHelper::builder() currently specializes in read operations)");

    // ========================================================================
    // Part 3: Process Operations
    // ========================================================================

    println!("\n\nPart 3: Process Operations with Security");
    println!("-----------------------------------------");

    // Create security middleware for process operations
    let process_acl = create_permissive_acl();
    let process_middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(process_acl))
        .build()?;

    // Create ProcessHelper with security
    let process_helper = ProcessHelper::builder().with_security(process_middleware);

    // Spawn a simple echo process
    println!("\n✓ Spawning echo process...");
    let output = process_helper
        .spawn(
            "echo",
            vec!["Hello from AirsSys OSL!".to_string()],
            "demo_user",
        )
        .await?;

    println!("  Process output: {}", String::from_utf8_lossy(&output));

    // ========================================================================
    // Part 4: Network Operations
    // ========================================================================

    println!("\n\nPart 4: Network Operations with Security");
    println!("-----------------------------------------");

    // Create security middleware for network operations
    let network_acl = create_permissive_acl();
    let network_middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(network_acl))
        .build()?;

    // Create NetworkHelper with security
    let network_helper = NetworkHelper::builder().with_security(network_middleware);

    // Connect to a well-known server
    println!("\n✓ Connecting to example.com:80...");
    match network_helper.connect("example.com:80", "demo_user").await {
        Ok(_) => println!("  Successfully connected to example.com:80"),
        Err(e) => println!("  Connection result: {e} (may fail if offline)"),
    }

    // ========================================================================
    // Cleanup and Summary
    // ========================================================================

    println!("\n\n=== Summary ===");
    println!("✓ Demonstrated file read operations with security");
    println!("✓ Showed how security policies enforce access control");
    println!("✓ Demonstrated process spawning with security");
    println!("✓ Demonstrated network connections with security");
    println!("\nKey Takeaways:");
    println!("- Composition API provides type-safe operation builders");
    println!("- SecurityMiddleware enforces policies transparently");
    println!("- Different helpers can use different security policies");
    println!("- All operations are audited and logged");

    // Cleanup
    fs::remove_file(demo_file).ok();
    println!("\n✓ Cleaned up demo file");

    Ok(())
}
