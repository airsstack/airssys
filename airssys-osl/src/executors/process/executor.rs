//! Process executor implementation.

// Layer 1: Standard library imports
use std::fmt;

/// Process executor for executing process management operations.
///
/// This executor provides real implementations for process operations using
/// tokio's async process management capabilities.
///
/// # Examples
///
/// ```rust
/// use airssys_osl::executors::ProcessExecutor;
///
/// let executor = ProcessExecutor::new("process-executor");
/// assert_eq!(executor.name(), "process-executor");
/// ```
#[derive(Debug, Clone)]
pub struct ProcessExecutor {
    /// Executor name for identification and logging
    pub(super) name: String,
}

impl ProcessExecutor {
    /// Create a new process executor.
    ///
    /// # Arguments
    ///
    /// * `name` - Executor name for identification
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::executors::ProcessExecutor;
    ///
    /// let executor = ProcessExecutor::new("my-process-executor");
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// Get the executor name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl fmt::Display for ProcessExecutor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ProcessExecutor({})", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let executor = ProcessExecutor::new("test-executor");
        assert_eq!(executor.name(), "test-executor");
    }

    #[test]
    fn test_display() {
        let executor = ProcessExecutor::new("my-executor");
        assert_eq!(executor.to_string(), "ProcessExecutor(my-executor)");
    }
}
