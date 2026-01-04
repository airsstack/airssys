//! Helper functions usage example for airssys-osl
//!
//! This example demonstrates the ergonomic helper function API for common operations.
//! Run with: cargo run --example helper_functions

#![allow(clippy::unwrap_used, clippy::expect_used)] // Allow in examples for clarity

use airssys_osl::helpers::*;
use airssys_osl::prelude::*;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> Result<(), OSError> {
    println!("=== AirsSys OSL Helper Functions Example ===\n");

    // Create temporary directory for examples
    let temp_dir = TempDir::new()
        .map_err(|e| OSError::execution_failed(format!("Failed to create temp directory: {e}")))?;
    let temp_path = temp_dir.path();

    // ==========================================
    // Filesystem Helper Functions
    // ==========================================
    println!("📁 Filesystem Operations");
    println!("========================\n");

    // 1. Write a file
    let file_path = temp_path.join("test.txt");
    println!("1. Writing file: {file_path:?}");
    write_file(
        file_path.to_string_lossy().to_string(),
        b"Hello, AirsSys OSL!".to_vec(),
        "example-user", // user parameter for security context
    )
    .await?;
    println!("   ✓ File written successfully\n");

    // 2. Read the file
    println!("2. Reading file: {file_path:?}");
    let content = read_file(file_path.to_string_lossy().to_string(), "example-user").await?;
    println!("   ✓ Read {} bytes", content.len());
    println!("   Content: {}", String::from_utf8_lossy(&content));
    println!();

    // 3. Create a directory
    let dir_path = temp_path.join("subdir");
    println!("3. Creating directory: {dir_path:?}");
    create_directory(dir_path.to_string_lossy().to_string(), "example-user").await?;
    println!("   ✓ Directory created successfully\n");

    // 4. Create a file in the subdirectory
    let nested_file = dir_path.join("nested.txt");
    println!("4. Creating nested file: {nested_file:?}");
    write_file(
        nested_file.to_string_lossy().to_string(),
        b"Nested file content".to_vec(),
        "example-user",
    )
    .await?;
    println!("   ✓ Nested file created successfully\n");

    // 5. Delete a file
    println!("5. Deleting file: {nested_file:?}");
    delete_file(nested_file.to_string_lossy().to_string(), "example-user").await?;
    println!("   ✓ File deleted successfully\n");

    // ==========================================
    // Process Helper Functions
    // ==========================================
    println!("⚙️ Process Operations");
    println!("======================\n");

    // 1. Spawn a process
    println!("1. Spawning process: echo 'Hello from process'");
    let output = spawn_process(
        "echo".to_string(),
        vec!["Hello from process".to_string()],
        "example-user",
    )
    .await?;
    println!("   ✓ Process spawned successfully\n");
    println!("   Output: {}", String::from_utf8_lossy(&output));

    // Wait a bit for the process to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // ==========================================
    // Network Helper Functions
    // ==========================================
    println!("🌐 Network Operations");
    println!("=====================\n");

    // 1. Create a socket
    println!("1. Creating TCP socket");
    create_socket("tcp".to_string(), "example-user").await?;
    println!("   ✓ TCP socket created successfully\n");

    // 2. Create a listener
    println!("2. Creating network listener on 127.0.0.1:0");
    let listener_result = network_listen("127.0.0.1:0".to_string(), "example-user").await;
    match listener_result {
        Ok(_) => println!("   ✓ Listener created successfully\n"),
        Err(e) => println!("   ⚠ Listener creation (expected in example): {e}\n"),
    }

    // 3. Connect to a network address (will fail in this example)
    println!("3. Attempting network connection to 127.0.0.1:8080");
    let connect_result = network_connect("127.0.0.1:8080".to_string(), "example-user").await;
    match connect_result {
        Ok(_) => println!("   ✓ Connected successfully\n"),
        Err(e) => println!("   ⚠ Connection failed (expected in example): {e}\n"),
    }

    // ==========================================
    // Summary
    // ==========================================
    println!("=== Summary ===");
    println!("✅ Demonstrated filesystem operations (write, read, create dir, delete)");
    println!("✅ Demonstrated process operations (spawn)");
    println!("✅ Demonstrated network operations (socket, listen, connect)");
    println!("\n🎉 All helper function examples completed successfully!");

    Ok(())
}
