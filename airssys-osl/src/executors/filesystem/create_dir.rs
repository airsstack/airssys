//! DirectoryCreateOperation executor implementation.
//!
//! Provides async directory creation using tokio::fs with support for
//! recursive directory creation.

use async_trait::async_trait;
use chrono::Utc;

use crate::core::context::ExecutionContext;
use crate::core::executor::{ExecutionResult, OSExecutor};
use crate::core::operation::OperationType;
use crate::core::result::{OSError, OSResult};
use crate::operations::filesystem::DirectoryCreateOperation;

use super::FilesystemExecutor;

#[async_trait]
impl OSExecutor<DirectoryCreateOperation> for FilesystemExecutor {
    fn name(&self) -> &str {
        &self.name
    }

    fn supported_operation_types(&self) -> Vec<OperationType> {
        vec![OperationType::Filesystem]
    }

    async fn execute(
        &self,
        operation: DirectoryCreateOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        let started_at = Utc::now();

        if operation.recursive {
            // Create directory and all parent directories
            tokio::fs::create_dir_all(&operation.path)
                .await
                .map_err(|e| {
                    OSError::filesystem_error("create_dir_all", &operation.path, e.to_string())
                })?;
        } else {
            // Create only the directory (parent must exist)
            tokio::fs::create_dir(&operation.path).await.map_err(|e| {
                OSError::filesystem_error("create_dir", &operation.path, e.to_string())
            })?;
        }

        let completed_at = Utc::now();

        let result = ExecutionResult::success_with_timing(Vec::new(), started_at, completed_at)
            .with_metadata("path".to_string(), operation.path.clone())
            .with_metadata("recursive".to_string(), operation.recursive.to_string())
            .with_metadata("executor".to_string(), self.name.to_string())
            .with_metadata("user".to_string(), context.principal().to_string());

        Ok(result)
    }

    async fn validate_operation(
        &self,
        operation: &DirectoryCreateOperation,
        _context: &ExecutionContext,
    ) -> OSResult<()> {
        // Check if directory already exists
        if tokio::fs::try_exists(&operation.path)
            .await
            .map_err(|e| OSError::filesystem_error("validate", &operation.path, e.to_string()))?
        {
            return Err(OSError::filesystem_error(
                "validate",
                &operation.path,
                "Directory already exists",
            ));
        }

        // If not recursive, validate parent directory exists
        if !operation.recursive {
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
                        "Parent directory does not exist (use recursive mode)",
                    ));
                }
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
    async fn test_directory_create_operation_success() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let dir_path = temp_dir.path().join("test_dir");
        let path = dir_path.to_str().expect("Invalid UTF-8 path").to_string();

        let executor = FilesystemExecutor::new();
        let operation = DirectoryCreateOperation::new(&path);
        let security_context = SecurityContext::new("test-user".to_string());
        let context = ExecutionContext::new(security_context);

        let result = executor
            .execute(operation, &context)
            .await
            .expect("Execution failed");
        assert_eq!(result.exit_code, 0);

        // Verify directory was created
        assert!(dir_path.exists());
        assert!(dir_path.is_dir());
    }
}
