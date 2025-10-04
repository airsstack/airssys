//! Executor registry for dynamic operation type dispatch.
//!
//! This module provides the ExecutorRegistry for managing and dispatching
//! operations to the appropriate executor based on operation type.
//!
//! Phase 2 Note: Since Operation trait has Clone bound (not object-safe),
//! we use a simplified registry that will be enhanced in Phase 3 with
//! concrete operation type support.

use std::collections::HashMap;

use crate::core::{operation::OperationType, result::OSResult};

/// Registry for managing operation executors with dynamic dispatch.
///
/// The ExecutorRegistry maintains a mapping of operation types to their
/// corresponding executors, enabling dynamic operation dispatch based on
/// the operation type.
///
/// # Phase 2 Implementation
///
/// This phase provides functional executor tracking. Full dynamic dispatch
/// with concrete operation types will be implemented in Phase 3.
///
/// # Examples
///
/// ```rust
/// use airssys_osl::framework::registry::ExecutorRegistry;
///
/// # fn example() -> airssys_osl::core::result::OSResult<()> {
/// let registry = ExecutorRegistry::new()?;
/// assert_eq!(registry.executor_count(), 0);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct ExecutorRegistry {
    /// Registered operation types and their executor names
    registered_types: HashMap<OperationType, String>,
}

impl ExecutorRegistry {
    /// Create a new executor registry.
    pub fn new() -> OSResult<Self> {
        Ok(Self {
            registered_types: HashMap::new(),
        })
    }

    /// Register an executor for specific operation types.
    ///
    /// # Arguments
    ///
    /// * `operation_types` - The types of operations this executor handles
    /// * `executor_name` - The name of the executor
    ///
    /// # Phase 2 Note
    ///
    /// Currently tracks executor names. Phase 3 will store actual executors
    /// when concrete operation types are implemented.
    pub fn register_executor(
        &mut self,
        operation_types: Vec<OperationType>,
        executor_name: &str,
    ) -> OSResult<()> {
        for op_type in operation_types {
            self.registered_types
                .insert(op_type, executor_name.to_string());
        }
        Ok(())
    }

    /// Check if an executor exists for the given operation type.
    pub fn has_executor(&self, operation_type: &OperationType) -> bool {
        self.registered_types.contains_key(operation_type)
    }

    /// Get the number of registered executors.
    pub fn executor_count(&self) -> usize {
        self.registered_types.len()
    }

    /// Get all registered operation types.
    pub fn registered_types(&self) -> Vec<OperationType> {
        self.registered_types.keys().copied().collect()
    }

    /// Get the executor name for a given operation type.
    ///
    /// # Phase 2 Note
    ///
    /// Returns the executor name. Phase 3 will return actual executor references.
    pub fn get_executor_name(&self, operation_type: &OperationType) -> Option<&str> {
        self.registered_types
            .get(operation_type)
            .map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)] // Allow in tests for clarity

    use super::*;

    #[test]
    fn test_registry_creation() {
        let result = ExecutorRegistry::new();
        assert!(result.is_ok());
        let registry = result.expect("Registry creation should succeed");
        assert_eq!(registry.executor_count(), 0);
    }

    #[test]
    fn test_has_executor_returns_false() {
        let registry = ExecutorRegistry::new().expect("Registry creation should succeed");
        assert!(!registry.has_executor(&OperationType::Filesystem));
        assert!(!registry.has_executor(&OperationType::Process));
    }

    #[test]
    fn test_registered_types_empty() {
        let registry = ExecutorRegistry::new().expect("Registry creation should succeed");
        let types = registry.registered_types();
        assert!(types.is_empty());
    }

    #[test]
    fn test_register_executor() {
        let mut registry = ExecutorRegistry::new().expect("Registry creation should succeed");

        let result = registry.register_executor(
            vec![OperationType::Filesystem, OperationType::Process],
            "test-executor",
        );

        assert!(result.is_ok());
        assert_eq!(registry.executor_count(), 2);
        assert!(registry.has_executor(&OperationType::Filesystem));
        assert!(registry.has_executor(&OperationType::Process));
        assert_eq!(
            registry.get_executor_name(&OperationType::Filesystem),
            Some("test-executor")
        );
    }

    #[test]
    fn test_get_executor_name() {
        let mut registry = ExecutorRegistry::new().expect("Registry creation should succeed");

        registry
            .register_executor(vec![OperationType::Network], "network-executor")
            .expect("Registration should succeed");

        assert_eq!(
            registry.get_executor_name(&OperationType::Network),
            Some("network-executor")
        );
        assert_eq!(registry.get_executor_name(&OperationType::Utility), None);
    }
}
