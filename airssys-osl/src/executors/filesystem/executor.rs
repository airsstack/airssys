//! FilesystemExecutor struct definition.
//!
//! Provides the core executor struct that implements OSExecutor trait
//! for various filesystem operation types.

/// Platform executor for filesystem operations.
///
/// Uses tokio::fs for async I/O operations. All operations include comprehensive
/// error handling, timing information, and metadata tracking.
///
/// # Example
///
/// ```rust
/// use airssys_osl::executors::FilesystemExecutor;
///
/// let executor = FilesystemExecutor::new();
/// assert_eq!(executor.name(), "filesystem-executor");
/// ```
#[derive(Debug, Clone)]
pub struct FilesystemExecutor {
    pub(super) name: String,
}

impl FilesystemExecutor {
    /// Create a new filesystem executor.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_osl::executors::FilesystemExecutor;
    ///
    /// let executor = FilesystemExecutor::new();
    /// assert_eq!(executor.name(), "filesystem-executor");
    /// ```
    pub fn new() -> Self {
        Self {
            name: "filesystem-executor".to_string(),
        }
    }

    /// Get the executor name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Default for FilesystemExecutor {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filesystem_executor_creation() {
        let executor = FilesystemExecutor::new();
        assert_eq!(executor.name(), "filesystem-executor");
    }

    #[test]
    fn test_filesystem_executor_default() {
        let executor = FilesystemExecutor::default();
        assert_eq!(executor.name(), "filesystem-executor");
    }
}
