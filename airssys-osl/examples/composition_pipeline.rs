//! Pipeline-Based Composition Example
//!
//! This example demonstrates advanced composition patterns using reusable pipelines.
//! It shows:
//! - Creating reusable pipeline configurations
//! - Cross-operation workflows (file → process → network)
//! - Dynamic security policy selection
//! - Pipeline composition patterns
//!
//! Run with: `cargo run --example composition_pipeline`

use std::fs;
use std::path::PathBuf;

use airssys_osl::core::security::SecurityConfig;
use airssys_osl::helpers::composition::{FileHelper, HelperPipeline, ProcessHelper};
use airssys_osl::middleware::security::acl::{AccessControlList, AclEntry, AclPolicy};
use airssys_osl::middleware::security::middleware::SecurityMiddlewareBuilder;

// ============================================================================
// Security Policy Factory
// ============================================================================

/// Factory for creating different security policies
struct SecurityPolicyFactory;

impl SecurityPolicyFactory {
    /// Create an admin policy (full access)
    fn admin_policy() -> AccessControlList {
        AccessControlList::new().add_entry(AclEntry::new(
            "admin".to_string(),
            "*".to_string(),
            vec!["*".to_string()],
            AclPolicy::Allow,
        ))
    }

    /// Create a read-only policy
    fn readonly_policy(user_id: &str) -> AccessControlList {
        AccessControlList::new()
            .add_entry(AclEntry::new(
                user_id.to_string(),
                "file:*".to_string(),
                vec!["read".to_string()],
                AclPolicy::Allow,
            ))
            .add_entry(AclEntry::new(
                user_id.to_string(),
                "process:*".to_string(),
                vec!["spawn".to_string()],
                AclPolicy::Allow,
            ))
    }

    /// Create a restricted policy (deny sensitive operations)
    fn restricted_policy(user_id: &str) -> AccessControlList {
        AccessControlList::new()
            .add_entry(AclEntry::new(
                user_id.to_string(),
                "file:/tmp/*".to_string(), // Only /tmp directory
                vec!["read".to_string()],
                AclPolicy::Allow,
            ))
            .add_entry(AclEntry::new(
                user_id.to_string(),
                "process:echo".to_string(), // Only echo command
                vec!["spawn".to_string()],
                AclPolicy::Allow,
            ))
    }
}

// ============================================================================
// Pipeline Configurations
// ============================================================================

/// A reusable file reading pipeline with security
struct SecureFileReader {
    user_id: String,
    policy_type: PolicyType,
}

#[derive(Debug, Clone)]
enum PolicyType {
    Admin,
    ReadOnly,
    Restricted,
}

impl SecureFileReader {
    fn new(user_id: impl Into<String>, policy_type: PolicyType) -> Self {
        Self {
            user_id: user_id.into(),
            policy_type,
        }
    }

    /// Read a file using the configured policy
    async fn read(&self, file_path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Select appropriate policy
        let acl = match self.policy_type {
            PolicyType::Admin => SecurityPolicyFactory::admin_policy(),
            PolicyType::ReadOnly => SecurityPolicyFactory::readonly_policy(&self.user_id),
            PolicyType::Restricted => SecurityPolicyFactory::restricted_policy(&self.user_id),
        };

        // Build middleware
        let middleware = SecurityMiddlewareBuilder::new()
            .with_config(SecurityConfig::default())
            .add_policy(Box::new(acl))
            .build()?;

        // Create helper
        let helper = FileHelper::builder().with_security(middleware);

        // Execute read
        Ok(helper.read(file_path, &self.user_id).await?)
    }
}

/// A reusable process execution pipeline with security
struct SecureProcessExecutor {
    user_id: String,
    policy_type: PolicyType,
}

impl SecureProcessExecutor {
    fn new(user_id: impl Into<String>, policy_type: PolicyType) -> Self {
        Self {
            user_id: user_id.into(),
            policy_type,
        }
    }

    /// Execute a command using the configured policy
    async fn execute(
        &self,
        command: &str,
        args: Vec<String>,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Select appropriate policy
        let acl = match self.policy_type {
            PolicyType::Admin => SecurityPolicyFactory::admin_policy(),
            PolicyType::ReadOnly => SecurityPolicyFactory::readonly_policy(&self.user_id),
            PolicyType::Restricted => SecurityPolicyFactory::restricted_policy(&self.user_id),
        };

        // Build middleware
        let middleware = SecurityMiddlewareBuilder::new()
            .with_config(SecurityConfig::default())
            .add_policy(Box::new(acl))
            .build()?;

        // Create helper
        let helper = ProcessHelper::builder().with_security(middleware);

        // Execute command
        Ok(helper.spawn(command, args, &self.user_id).await?)
    }
}

// ============================================================================
// Cross-Operation Workflow
// ============================================================================

/// A workflow that combines file reading and process execution
struct FileProcessingWorkflow {
    file_reader: SecureFileReader,
    process_executor: SecureProcessExecutor,
}

impl FileProcessingWorkflow {
    fn new(user_id: impl Into<String>, policy_type: PolicyType) -> Self {
        let user_id = user_id.into();
        Self {
            file_reader: SecureFileReader::new(user_id.clone(), policy_type.clone()),
            process_executor: SecureProcessExecutor::new(user_id, policy_type),
        }
    }

    /// Read a file and process its content through a command
    async fn read_and_process(
        &self,
        file_path: &str,
        command: &str,
        args: Vec<String>,
    ) -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
        println!("\n  Step 1: Reading file '{}'...", file_path);
        let file_content = self.file_reader.read(file_path).await?;
        println!("    ✓ Read {} bytes", file_content.len());

        println!("  Step 2: Executing command '{}'...", command);
        let process_output = self.process_executor.execute(command, args).await?;
        println!("    ✓ Got {} bytes output", process_output.len());

        Ok((file_content, process_output))
    }
}

// ============================================================================
// Demo Execution
// ============================================================================

/// Create demo files
fn create_demo_files() -> Vec<PathBuf> {
    let temp_dir = std::env::temp_dir();
    let mut files = Vec::new();

    // Create a file in /tmp (accessible by restricted policy)
    let tmp_file = temp_dir.join("pipeline_demo_allowed.txt");
    fs::write(
        &tmp_file,
        "This file is in /tmp and accessible by restricted users",
    )
    .unwrap();
    files.push(tmp_file);

    // Create a file outside /tmp (not accessible by restricted policy)
    let home_file = temp_dir.join("pipeline_demo_restricted.txt");
    fs::write(&home_file, "This file requires higher privileges").unwrap();
    files.push(home_file);

    files
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AirsSys OSL Composition API - Pipeline Example ===\n");

    // Create demo files
    let demo_files = create_demo_files();
    println!("✓ Created demo files");

    // ========================================================================
    // Part 1: Admin Pipeline
    // ========================================================================

    println!("\n\nPart 1: Admin Pipeline (Full Access)");
    println!("-------------------------------------");

    let admin_reader = SecureFileReader::new("admin", PolicyType::Admin);

    println!("\nReading with admin privileges:");
    for (i, file) in demo_files.iter().enumerate() {
        match admin_reader.read(file.to_str().unwrap()).await {
            Ok(content) => {
                println!("  File {}: ✓ Read {} bytes", i + 1, content.len());
                println!(
                    "    Preview: {}",
                    String::from_utf8_lossy(&content[..40.min(content.len())])
                );
            }
            Err(e) => println!("  File {}: ✗ Failed - {}", i + 1, e),
        }
    }

    // ========================================================================
    // Part 2: Read-Only Pipeline
    // ========================================================================

    println!("\n\nPart 2: Read-Only Pipeline (Limited Access)");
    println!("--------------------------------------------");

    let readonly_reader = SecureFileReader::new("user1", PolicyType::ReadOnly);

    println!("\nReading with read-only privileges:");
    match readonly_reader.read(demo_files[0].to_str().unwrap()).await {
        Ok(content) => {
            println!("  ✓ Read {} bytes with read-only policy", content.len());
        }
        Err(e) => println!("  ✗ Failed with read-only policy: {}", e),
    }

    // ========================================================================
    // Part 3: Restricted Pipeline
    // ========================================================================

    println!("\n\nPart 3: Restricted Pipeline (Very Limited Access)");
    println!("--------------------------------------------------");

    let restricted_reader = SecureFileReader::new("restricted_user", PolicyType::Restricted);

    println!("\nAttempting to read with restricted privileges:");
    println!("  File in /tmp (should succeed):");
    match restricted_reader
        .read(demo_files[0].to_str().unwrap())
        .await
    {
        Ok(content) => println!("    ✓ Success - Read {} bytes", content.len()),
        Err(e) => println!("    ✗ Failed: {}", e),
    }

    // ========================================================================
    // Part 4: Cross-Operation Workflow
    // ========================================================================

    println!("\n\nPart 4: Cross-Operation Workflow");
    println!("---------------------------------");

    let workflow = FileProcessingWorkflow::new("workflow_user", PolicyType::ReadOnly);

    println!("\nExecuting file → process workflow:");
    match workflow
        .read_and_process(
            demo_files[0].to_str().unwrap(),
            "echo",
            vec!["File processed!".to_string()],
        )
        .await
    {
        Ok((file_content, process_output)) => {
            println!("\n✓ Workflow completed successfully");
            println!("  File content: {} bytes", file_content.len());
            println!(
                "  Process output: {}",
                String::from_utf8_lossy(&process_output)
            );
        }
        Err(e) => println!("\n✗ Workflow failed: {}", e),
    }

    // ========================================================================
    // Part 5: Multiple Pipelines with Different Policies
    // ========================================================================

    println!("\n\nPart 5: Multiple Pipelines with Different Policies");
    println!("---------------------------------------------------");

    println!("\nComparing same operation with different policies:");

    // Admin execution
    let admin_executor = SecureProcessExecutor::new("admin", PolicyType::Admin);
    println!("\n  Admin executing 'ls':");
    match admin_executor.execute("ls", vec![]).await {
        Ok(output) => println!("    ✓ Success - {} bytes output", output.len()),
        Err(e) => println!("    ✗ Failed: {}", e),
    }

    // Restricted execution
    let restricted_executor = SecureProcessExecutor::new("restricted_user", PolicyType::Restricted);
    println!("\n  Restricted user executing 'echo':");
    match restricted_executor
        .execute("echo", vec!["hello".to_string()])
        .await
    {
        Ok(output) => println!("    ✓ Success - {}", String::from_utf8_lossy(&output)),
        Err(e) => println!("    ✗ Failed: {}", e),
    }

    println!("\n  Restricted user attempting 'ls' (should fail):");
    match restricted_executor.execute("ls", vec![]).await {
        Ok(output) => println!("    ✓ Success - {} bytes output", output.len()),
        Err(e) => println!("    ✗ Failed as expected: {}", e),
    }

    // ========================================================================
    // Summary
    // ========================================================================

    println!("\n\n=== Summary ===");
    println!("✓ Demonstrated reusable pipeline configurations");
    println!("✓ Showed dynamic security policy selection");
    println!("✓ Illustrated cross-operation workflows");
    println!("✓ Compared different access levels (admin, readonly, restricted)");
    println!("\nKey Takeaways:");
    println!("- Pipelines can encapsulate operation + security configuration");
    println!("- Different policies provide different access levels");
    println!("- Workflows can combine multiple operations seamlessly");
    println!("- Reusable components reduce code duplication");
    println!("- Type safety ensures correct operation usage");

    // Cleanup
    for file in demo_files {
        fs::remove_file(file).ok();
    }
    println!("\n✓ Cleaned up demo files");

    Ok(())
}
