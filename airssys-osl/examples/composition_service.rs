//! Service-Oriented Composition Example
//!
//! This example demonstrates a more complex, service-oriented application pattern
//! using the composition API. It shows:
//! - Building a file processing service with multiple security policies
//! - Chaining custom middleware with security middleware
//! - Handling errors gracefully in a service context
//! - Processing multiple files with a reusable pipeline
//!
//! Run with: `cargo run --example composition_service`

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use airssys_osl::core::context::ExecutionContext;
use airssys_osl::core::executor::ExecutionResult;
use airssys_osl::core::middleware::{Middleware, MiddlewareError, MiddlewareResult};
use airssys_osl::core::result::OSResult;
use airssys_osl::core::security::SecurityConfig;
use airssys_osl::helpers::composition::{FileHelper, HelperPipeline};
use airssys_osl::middleware::security::acl::{AccessControlList, AclEntry, AclPolicy};
use airssys_osl::middleware::security::middleware::SecurityMiddlewareBuilder;
use airssys_osl::operations::filesystem::FileReadOperation;

// ============================================================================
// Custom Middleware: Audit Logger
// ============================================================================

/// Audit logging middleware that tracks all file operations
#[derive(Debug, Clone)]
struct AuditLogger {
    log: Arc<Mutex<Vec<String>>>,
}

impl AuditLogger {
    fn new() -> Self {
        Self {
            log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_logs(&self) -> Vec<String> {
        self.log.lock().unwrap().clone()
    }
}

#[async_trait]
impl Middleware<FileReadOperation> for AuditLogger {
    fn name(&self) -> &str {
        "AuditLogger"
    }

    async fn before_execution(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> MiddlewareResult<Option<FileReadOperation>> {
        // Log the operation details
        let log_entry = format!(
            "[AUDIT] User '{}' attempting to read file: {}",
            context.principal(),
            operation.path
        );
        self.log.lock().unwrap().push(log_entry.clone());
        println!("  {}", log_entry);

        Ok(Some(operation))
    }

    async fn after_execution(
        &self,
        context: &ExecutionContext,
        result: &OSResult<ExecutionResult>,
    ) -> MiddlewareResult<()> {
        // Log the result
        let log_entry = match result {
            Ok(_) => format!(
                "[AUDIT] User '{}' - Operation succeeded",
                context.principal()
            ),
            Err(e) => format!(
                "[AUDIT] User '{}' - Operation failed: {}",
                context.principal(),
                e
            ),
        };
        self.log.lock().unwrap().push(log_entry.clone());
        println!("  {}", log_entry);

        Ok(())
    }
}

// ============================================================================
// Custom Middleware: File Size Validator
// ============================================================================

/// Middleware that validates file sizes before reading
#[derive(Debug, Clone)]
struct FileSizeValidator {
    max_size_bytes: u64,
}

impl FileSizeValidator {
    fn new(max_size_bytes: u64) -> Self {
        Self { max_size_bytes }
    }
}

#[async_trait]
impl Middleware<FileReadOperation> for FileSizeValidator {
    fn name(&self) -> &str {
        "FileSizeValidator"
    }

    async fn before_execution(
        &self,
        operation: FileReadOperation,
        _context: &ExecutionContext,
    ) -> MiddlewareResult<Option<FileReadOperation>> {
        // Check file size
        if let Ok(metadata) = fs::metadata(&operation.path) {
            let size = metadata.len();
            if size > self.max_size_bytes {
                println!(
                    "  [SIZE_VALIDATOR] File too large: {} bytes (max: {} bytes)",
                    size, self.max_size_bytes
                );
                return Err(MiddlewareError::Fatal(format!(
                    "File exceeds maximum allowed size: {} bytes (max: {} bytes)",
                    size, self.max_size_bytes
                )));
            }
            println!(
                "  [SIZE_VALIDATOR] File size OK: {} bytes (max: {} bytes)",
                size, self.max_size_bytes
            );
        }

        Ok(Some(operation))
    }

    async fn after_execution(
        &self,
        _context: &ExecutionContext,
        _result: &OSResult<ExecutionResult>,
    ) -> MiddlewareResult<()> {
        Ok(())
    }
}

// ============================================================================
// File Processing Service
// ============================================================================

/// A service that processes files with comprehensive security and validation
struct FileProcessingService {
    audit_logger: AuditLogger,
}

impl FileProcessingService {
    fn new() -> Self {
        Self {
            audit_logger: AuditLogger::new(),
        }
    }

    /// Process a single file with full security and validation
    async fn process_file(
        &self,
        file_path: &Path,
        user_id: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        println!("\nProcessing file: {}", file_path.display());

        // Create ACL that allows this user to read files
        let acl = AccessControlList::new().add_entry(AclEntry::new(
            user_id.to_string(),
            "file:*".to_string(),
            vec!["read".to_string()],
            AclPolicy::Allow,
        ));

        // Build security middleware
        let security = SecurityMiddlewareBuilder::new()
            .with_config(SecurityConfig::default())
            .add_policy(Box::new(acl))
            .build()?;

        // Create file size validator (max 1MB for this example)
        let size_validator = FileSizeValidator::new(1024 * 1024);

        // Build the processing pipeline with multiple middleware
        let helper = FileHelper::builder()
            .with_security(security)
            .with_middleware(self.audit_logger.clone())
            .with_middleware(size_validator);

        // Execute the read operation
        let content = helper.read(file_path.to_str().unwrap(), user_id).await?;

        println!("  ✓ Successfully processed {} bytes", content.len());
        Ok(content)
    }

    /// Process multiple files in batch
    async fn process_batch(
        &self,
        file_paths: &[PathBuf],
        user_id: &str,
    ) -> Vec<Result<Vec<u8>, Box<dyn std::error::Error>>> {
        println!("\n=== Batch Processing {} files ===", file_paths.len());

        let mut results = Vec::new();
        for path in file_paths {
            let result = self.process_file(path, user_id).await;
            results.push(result);
        }

        results
    }

    /// Get audit logs
    fn get_audit_logs(&self) -> Vec<String> {
        self.audit_logger.get_logs()
    }
}

// ============================================================================
// Demo Setup and Execution
// ============================================================================

/// Create demo files for testing
fn create_demo_files() -> Vec<PathBuf> {
    let temp_dir = std::env::temp_dir();
    let mut files = Vec::new();

    // Create small file (will pass size validation)
    let small_file = temp_dir.join("service_demo_small.txt");
    fs::write(
        &small_file,
        "Small file content - this will pass validation",
    )
    .unwrap();
    files.push(small_file);

    // Create medium file (will pass size validation)
    let medium_file = temp_dir.join("service_demo_medium.txt");
    let medium_content = "Medium file content. ".repeat(100); // ~2KB
    fs::write(&medium_file, medium_content).unwrap();
    files.push(medium_file);

    // Create another small file
    let another_small = temp_dir.join("service_demo_another.txt");
    fs::write(&another_small, "Another small file for batch processing").unwrap();
    files.push(another_small);

    files
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AirsSys OSL Composition API - Service Example ===\n");

    // Create the file processing service
    let service = FileProcessingService::new();
    println!("✓ File Processing Service initialized");

    // Create demo files
    let demo_files = create_demo_files();
    println!("✓ Created {} demo files", demo_files.len());

    // ========================================================================
    // Part 1: Single File Processing
    // ========================================================================

    println!("\n\nPart 1: Single File Processing");
    println!("-------------------------------");

    match service.process_file(&demo_files[0], "service_user").await {
        Ok(content) => {
            println!("\n✓ File processed successfully");
            println!(
                "  Content preview: {}",
                String::from_utf8_lossy(&content[..50.min(content.len())])
            );
        }
        Err(e) => {
            println!("\n✗ File processing failed: {}", e);
        }
    }

    // ========================================================================
    // Part 2: Batch Processing
    // ========================================================================

    println!("\n\nPart 2: Batch File Processing");
    println!("------------------------------");

    let results = service.process_batch(&demo_files, "service_user").await;

    println!("\n=== Batch Processing Results ===");
    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(content) => println!("  File {}: ✓ Processed {} bytes", i + 1, content.len()),
            Err(e) => println!("  File {}: ✗ Failed - {}", i + 1, e),
        }
    }

    // ========================================================================
    // Part 3: Audit Log Review
    // ========================================================================

    println!("\n\nPart 3: Audit Log Review");
    println!("------------------------");

    let logs = service.get_audit_logs();
    println!("\nTotal audit log entries: {}", logs.len());
    println!("\nAudit log:");
    for (i, log) in logs.iter().enumerate() {
        println!("  {}. {}", i + 1, log);
    }

    // ========================================================================
    // Summary
    // ========================================================================

    println!("\n\n=== Summary ===");
    println!("✓ Demonstrated service-oriented architecture");
    println!("✓ Showed custom middleware integration (audit + validation)");
    println!(
        "✓ Processed {} files with security enforcement",
        demo_files.len()
    );
    println!("✓ Generated {} audit log entries", logs.len());
    println!("\nKey Takeaways:");
    println!("- Services can encapsulate complex processing logic");
    println!("- Multiple middleware can be chained for layered functionality");
    println!("- Audit logging provides comprehensive operation tracking");
    println!("- Validation middleware can enforce business rules");
    println!("- Reusable pipelines enable consistent processing");

    // Cleanup
    for file in demo_files {
        fs::remove_file(file).ok();
    }
    println!("\n✓ Cleaned up demo files");

    Ok(())
}
