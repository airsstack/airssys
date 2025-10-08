//! FileReadOperation executor implementation.
//!
//! Provides async file reading using tokio::fs with comprehensive error handling
//! and timing information.

use async_trait::async_trait;
use chrono::Utc;

use crate::core::context::ExecutionContext;
use crate::core::executor::{ExecutionResult, OSExecutor};
use crate::core::operation::OperationType;
use crate::core::result::{OSError, OSResult};
use crate::operations::filesystem::FileReadOperation;

use super::FilesystemExecutor;

#[async_trait]
impl OSExecutor<FileReadOperation> for FilesystemExecutor {
    fn name(&self) -> &str {
        &self.name
    }

    fn supported_operation_types(&self) -> Vec<OperationType> {
        vec![OperationType::Filesystem]
    }

    async fn execute(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        let started_at = Utc::now();

        // Read file using tokio::fs
        let content = tokio::fs::read(&operation.path)
            .await
            .map_err(|e| OSError::filesystem_error("read", &operation.path, e.to_string()))?;

        let completed_at = Utc::now();

        // Create result with timing information
        let result = ExecutionResult::success_with_timing(content, started_at, completed_at)
            .with_metadata("path".to_string(), operation.path.clone())
            .with_metadata("executor".to_string(), self.name.to_string())
            .with_metadata("user".to_string(), context.principal().to_string());

        Ok(result)
    }

    async fn validate_operation(
        &self,
        operation: &FileReadOperation,
        _context: &ExecutionContext,
    ) -> OSResult<()> {
        // Validate file exists
        if !tokio::fs::try_exists(&operation.path)
            .await
            .map_err(|e| OSError::filesystem_error("validate", &operation.path, e.to_string()))?
        {
            return Err(OSError::filesystem_error(
                "validate",
                &operation.path,
                "File does not exist",
            ));
        }

        // Validate path is not a directory
        let metadata = tokio::fs::metadata(&operation.path)
            .await
            .map_err(|e| OSError::filesystem_error("validate", &operation.path, e.to_string()))?;

        if metadata.is_dir() {
            return Err(OSError::filesystem_error(
                "validate",
                &operation.path,
                "Path is a directory, not a file",
            ));
        }

        Ok(())
    }

    async fn cleanup(&self, _context: &ExecutionContext) -> OSResult<()> {
        // No cleanup needed for filesystem reads
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::core::context::SecurityContext;

    #[tokio::test]
    async fn test_file_read_operation_success() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(b"Hello, World!")
            .expect("Failed to write to temp file");
        let path = temp_file
            .path()
            .to_str()
            .expect("Invalid UTF-8 path")
            .to_string();

        let executor = FilesystemExecutor::new();
        let operation = FileReadOperation::new(&path);
        let security_context = SecurityContext::new("test-user".to_string());
        let context = ExecutionContext::new(security_context);

        let result = executor
            .execute(operation, &context)
            .await
            .expect("Execution failed");
        assert_eq!(&result.output, b"Hello, World!");
        assert_eq!(result.exit_code, 0);
    }
}
