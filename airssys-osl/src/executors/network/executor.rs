//! NetworkExecutor struct definition.

// Layer 1: Standard library imports
use std::fmt;

/// Network executor for handling network operations.
///
/// This executor implements the `OSExecutor` trait for network operations including
/// TCP/UDP connections, listeners, and socket creation. It uses tokio's async
/// networking primitives for all operations.
///
/// # Examples
///
/// ```rust
/// use airssys_osl::executors::NetworkExecutor;
///
/// let executor = NetworkExecutor::new("network-executor");
/// ```
#[derive(Debug, Clone)]
pub struct NetworkExecutor {
    /// Name of this executor instance
    pub(super) name: String,
}

impl NetworkExecutor {
    /// Create a new network executor with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - A name for this executor instance (used in logging and metadata)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::executors::NetworkExecutor;
    ///
    /// let executor = NetworkExecutor::new("my-network-executor");
    /// ```
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { name: name.into() }
    }
}

impl fmt::Display for NetworkExecutor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NetworkExecutor({})", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_executor_creation() {
        let executor = NetworkExecutor::new("test-executor");
        assert_eq!(executor.name, "test-executor");
    }

    #[test]
    fn test_network_executor_display() {
        let executor = NetworkExecutor::new("my-executor");
        assert_eq!(executor.to_string(), "NetworkExecutor(my-executor)");
    }
}
