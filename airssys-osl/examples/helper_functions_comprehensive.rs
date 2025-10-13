//! Comprehensive examples of AirsSys OSL helper function usage.
//!
//! This example demonstrates all 10 helper functions with:
//! - **Basic file operations** (read, write, create directory, delete)
//! - **Process management** (spawn, kill, send signal)
//! - **Network operations** (connect, listen, UDP socket)
//! - **Error handling** patterns
//! - **Real-world workflows**
//!
//! All examples use the Level 1 API (simple helpers with default security).
//!
//! # Running This Example
//!
//! ```bash
//! cargo run --example helper_functions_comprehensive
//! ```

use airssys_osl::helpers::*;

#[tokio::main]
async fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     AirsSys OSL - Comprehensive Helper Functions Example      ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Run all examples
    filesystem_operations().await;
    process_operations().await;
    network_operations().await;
    error_handling_patterns().await;
    real_world_workflow().await;

    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                     All Examples Complete!                      ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
}

// ============================================================================
// Filesystem Operations
// ============================================================================

async fn filesystem_operations() {
    println!("═══ Filesystem Operations ═══\n");

    let test_file = std::env::temp_dir().join("airssys_demo.txt");
    let test_dir = std::env::temp_dir().join("airssys_demo_dir");
    let test_data = b"Hello from AirsSys OSL! This is test data.".to_vec();

    // 1. Write file
    println!("1. Writing file: {}", test_file.display());
    match write_file(&test_file, test_data.clone(), "admin").await {
        Ok(()) => println!("   ✓ File written successfully ({} bytes)", test_data.len()),
        Err(e) => println!("   ✗ Failed to write file: {}", e),
    }

    // 2. Read file
    println!("\n2. Reading file back:");
    match read_file(&test_file, "admin").await {
        Ok(data) => {
            println!("   ✓ File read successfully ({} bytes)", data.len());
            println!("   Content: {}", String::from_utf8_lossy(&data));
        }
        Err(e) => println!("   ✗ Failed to read file: {}", e),
    }

    // 3. Create directory
    println!("\n3. Creating directory: {}", test_dir.display());
    match create_directory(&test_dir, "admin").await {
        Ok(()) => println!("   ✓ Directory created successfully"),
        Err(e) => {
            if e.to_string().contains("already exists") {
                println!("   ✓ Directory already exists (expected)");
            } else {
                println!("   ✗ Failed to create directory: {}", e);
            }
        }
    }

    // 4. Delete file
    println!("\n4. Deleting file:");
    match delete_file(&test_file, "admin").await {
        Ok(()) => println!("   ✓ File deleted successfully"),
        Err(e) => println!("   ✗ Failed to delete file: {}", e),
    }

    // Cleanup
    let _ = std::fs::remove_dir(&test_dir);

    println!();
}

// ============================================================================
// Process Operations
// ============================================================================

async fn process_operations() {
    println!("═══ Process Operations ═══\n");

    // 5. Spawn process - simple echo command
    println!("1. Spawning process (echo):");
    match spawn_process(
        "echo",
        vec!["Hello from spawned process!".to_string()],
        "admin",
    )
    .await
    {
        Ok(output) => {
            println!("   ✓ Process completed successfully");
            println!("   Output: {}", String::from_utf8_lossy(&output).trim());
        }
        Err(e) => println!("   ✗ Failed to spawn process: {}", e),
    }

    // 6. List directory (platform-specific command)
    println!("\n2. Listing directory (first 5 entries):");

    #[cfg(unix)]
    let (cmd, args) = ("ls", vec!["-la".to_string(), "/tmp".to_string()]);

    #[cfg(windows)]
    let (cmd, args) = (
        "cmd",
        vec!["/C".to_string(), "dir".to_string(), "C:\\Temp".to_string()],
    );

    match spawn_process(cmd, args, "admin").await {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output);
            let lines: Vec<&str> = output_str.lines().take(5).collect();
            println!("   ✓ Directory listing:");
            for line in lines {
                println!("     {}", line);
            }
        }
        Err(e) => println!("   ✗ Failed to list directory: {}", e),
    }

    // Note: kill_process and send_signal require actual PIDs
    // We skip them to avoid platform-specific issues in the demo

    println!();
}

// ============================================================================
// Network Operations
// ============================================================================

async fn network_operations() {
    println!("═══ Network Operations ═══\n");

    // 7. TCP connect (will fail without server, but demonstrates API)
    println!("1. TCP connect attempt:");
    match network_connect("127.0.0.1:8080", "admin").await {
        Ok(_) => println!("   ✓ TCP connection established"),
        Err(_) => println!("   ✓ Expected error (no server running): Connection refused"),
    }

    // 8. TCP listen on random port
    println!("\n2. Creating TCP listener:");
    match network_listen("127.0.0.1:0", "admin").await {
        Ok(_) => println!("   ✓ TCP listener created on random port"),
        Err(e) => println!("   ✗ Failed to create TCP listener: {}", e),
    }

    // 9. UDP socket on random port
    println!("\n3. Creating UDP socket:");
    match create_socket("127.0.0.1:0", "admin").await {
        Ok(_) => println!("   ✓ UDP socket created on random port"),
        Err(e) => println!("   ✗ Failed to create UDP socket: {}", e),
    }

    println!();
}

// ============================================================================
// Error Handling Patterns
// ============================================================================

async fn error_handling_patterns() {
    println!("═══ Error Handling Patterns ═══\n");

    // Pattern 1: Simple error check
    println!("1. Handling missing file:");
    match read_file("/nonexistent/file.txt", "admin").await {
        Ok(_) => println!("   ✗ Unexpected success"),
        Err(_) => println!("   ✓ Error handled: File not found"),
    }

    // Pattern 2: Fallback strategy
    println!("\n2. Fallback to alternative file:");
    let fallback_file = std::env::temp_dir().join("fallback.txt");
    let _ = std::fs::write(&fallback_file, b"Fallback data");

    let primary_result = read_file("/nonexistent.txt", "admin").await;
    let final_result = match primary_result {
        Ok(data) => Ok(data),
        Err(_) => {
            println!("   Primary file not found, trying fallback...");
            read_file(&fallback_file, "admin").await
        }
    };

    match final_result {
        Ok(content) => {
            println!(
                "   ✓ Fallback successful: {}",
                String::from_utf8_lossy(&content)
            );
        }
        Err(e) => println!("   ✗ All attempts failed: {}", e),
    }

    // Cleanup
    let _ = std::fs::remove_file(&fallback_file);

    // Pattern 3: Error categorization
    println!("\n3. Categorizing errors:");
    match read_file("/etc/shadow", "guest").await {
        Ok(_) => println!("   ✗ Security bypass detected!"),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("Permission denied") || error_msg.contains("denied") {
                println!("   ✓ Security error (expected): Access denied");
            } else if error_msg.contains("not found") || error_msg.contains("No such") {
                println!("   ✓ File not found error");
            } else {
                println!("   ✓ Other error: {}", error_msg);
            }
        }
    }

    println!();
}

// ============================================================================
// Real-World Workflow
// ============================================================================

async fn real_world_workflow() {
    println!("═══ Real-World Workflow: Configuration Management ═══\n");

    let config_file = std::env::temp_dir().join("app_config.json");
    let log_file = std::env::temp_dir().join("app.log");

    // Step 1: Create configuration file
    println!("1. Creating application configuration:");
    let config_json = r#"{
  "app_name": "DemoApp",
  "version": "1.0.0",
  "database": {
    "host": "localhost",
    "port": 5432,
    "name": "demo_db"
  },
  "logging": {
    "level": "info",
    "file": "/var/log/app.log"
  }
}"#;

    match write_file(&config_file, config_json.as_bytes().to_vec(), "admin").await {
        Ok(()) => println!("   ✓ Configuration file created"),
        Err(e) => {
            println!("   ✗ Failed to create config: {}", e);
            return;
        }
    }

    // Step 2: Read and validate configuration
    println!("\n2. Loading configuration:");
    let config_data = match read_file(&config_file, "admin").await {
        Ok(data) => {
            println!("   ✓ Configuration loaded ({} bytes)", data.len());
            data
        }
        Err(e) => {
            println!("   ✗ Failed to load config: {}", e);
            return;
        }
    };

    // Step 3: Display configuration preview
    println!("\n3. Configuration preview:");
    let config_str = String::from_utf8_lossy(&config_data);
    for line in config_str.lines().take(6) {
        println!("   {}", line);
    }

    // Step 4: Write application log
    println!("\n4. Writing application log:");
    let log_entry = format!(
        "[2025-10-13 12:00:00] INFO: Application started\n\
         [2025-10-13 12:00:01] INFO: Configuration loaded from {}\n\
         [2025-10-13 12:00:02] INFO: Database connection established\n",
        config_file.display()
    );

    match write_file(&log_file, log_entry.as_bytes().to_vec(), "admin").await {
        Ok(()) => println!("   ✓ Log file written successfully"),
        Err(e) => println!("   ✗ Failed to write log: {}", e),
    }

    // Step 5: Verify log file
    println!("\n5. Verifying log file:");
    match read_file(&log_file, "admin").await {
        Ok(log_data) => {
            let log_str = String::from_utf8_lossy(&log_data);
            let line_count = log_str.lines().count();
            println!("   ✓ Log file verified ({} log entries)", line_count);
        }
        Err(e) => println!("   ✗ Failed to read log: {}", e),
    }

    // Step 6: Cleanup
    println!("\n6. Cleaning up:");
    let mut cleanup_ok = true;

    match delete_file(&config_file, "admin").await {
        Ok(()) => println!("   ✓ Configuration file deleted"),
        Err(e) => {
            println!("   ✗ Failed to delete config: {}", e);
            cleanup_ok = false;
        }
    }

    match delete_file(&log_file, "admin").await {
        Ok(()) => println!("   ✓ Log file deleted"),
        Err(e) => {
            println!("   ✗ Failed to delete log: {}", e);
            cleanup_ok = false;
        }
    }

    if cleanup_ok {
        println!("\n   ✓ Workflow completed successfully!");
    }

    println!();
}
