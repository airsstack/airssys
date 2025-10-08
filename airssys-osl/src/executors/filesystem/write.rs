//! FileWriteOperation executor implementation.
//!
//! Provides async file writing using tokio::fs with support for both
//! append and overwrite modes.

use async_trait::async_trait;
use chrono::Utc;
use tokio::io::AsyncWriteExt;

use crate::core::context::ExecutionContext;
use crate::core::executor::{ExecutionResult, OSExecutor};
use crate::core::operation::OperationType;
use crate::core::result::{OSError, OSResult};
use crate::operations::filesystem::FileWriteOperation;

use super::FilesystemExecutor;

#[async_trait]
impl OSExecutor<FileWriteOperation> for FilesystemExecutor {
    fn name(&self) -> &str {
        &self.name
    }

    fn supported_operation_types(&self) -> Vec<OperationType> {
        vec![OperationType::Filesystem]
    }

    async fn execute(
        &self,
        operation: FileWriteOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        let started_at = Utc::now();

        if operation.append {
            // Append mode
            let mut file = tokio::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&operation.path)
                .await
                .map_err(|e| {
                    OSError::filesystem_error("open_append", &operation.path, e.to_string())
                })?;

            file.write_all(&operation.content).await.map_err(|e| {
                OSError::filesystem_error("write_append", &operation.path, e.to_string())
            })?;

            file.flush()
                .await
                .map_err(|e| OSError::filesystem_error("flush", &operation.path, e.to_string()))?;
        } else {
            // Overwrite mode
            tokio::fs::write(&operation.path, &operation.content)
                .await
                .map_err(|e| OSError::filesystem_error("write", &operation.path, e.to_string()))?;
        }

        let completed_at = Utc::now();

        let result = ExecutionResult::success_with_timing(Vec::new(), started_at, completed_at)
            .with_metadata("path".to_string(), operation.path.clone())
            .with_metadata(
                "bytes_written".to_string(),
                operation.content.len().to_string(),
            )
            .with_metadata(
                "mode".to_string(),
                if operation.append {
                    "append"
                } else {
                    "overwrite"
                }
                .to_string(),
            )
            .with_metadata("executor".to_string(), self.name.to_string())
            .with_metadata("user".to_string(), context.principal().to_string());

        Ok(result)
    }

    async fn validate_operation(
        &self,
        operation: &FileWriteOperation,
        _context: &ExecutionContext,
    ) -> OSResult<()> {
        // Validate parent directory exists
        if let Some(parent) = std::path::Path::new(&operation.path).parent() {
            if !tokio::fs::try_exists(parent).await.map_err(|e| {
                OSError::filesystem_error(
                    "validate",
                    &operation.path,
                    format!("Cannot check parent directory: {e}"),
                )
            })? {
                return Err(OSError::filesystem_error(
                    "validate",
                    &operation.path,
                    "Parent directory does not exist",
                ));
            }
        }

        Ok(())
    }

    async fn cleanup(&self, _context: &ExecutionContext) -> OSResult<()> {
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
    async fn test_file_write_operation_success() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.txt");
        let path = file_path.to_str().expect("Invalid UTF-8 path").to_string();

        let executor = FilesystemExecutor::new();
        let operation = FileWriteOperation::new(&path, b"Test content".to_vec());
        let security_context = SecurityContext::new("test-user".to_string());
        let context = ExecutionContext::new(security_context);

        let result = executor
            .execute(operation, &context)
            .await
            .expect("Execution failed");
        assert_eq!(result.exit_code, 0);

        // Verify file was written
        let content = std::fs::read_to_string(&file_path).expect("Failed to read file");
        assert_eq!(content, "Test content");
    }
}
