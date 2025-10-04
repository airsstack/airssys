//! Middleware pipeline orchestration for operation processing.
//!
//! This module provides the MiddlewarePipeline for managing and executing
//! middleware in sequence before, during, and after operation execution.
//!
//! Phase 2 provides simplified middleware tracking. Full middleware execution
//! with dynamic dispatch will be completed in Phase 3 when concrete operation
//! implementations are available.

/// Middleware pipeline for orchestrating operation processing.
///
/// The MiddlewarePipeline manages a collection of middleware components and
/// executes them in sequence during operation processing.
///
/// # Phase 2 Implementation
///
/// This phase provides the foundational pipeline structure with lifecycle
/// management. Full middleware orchestration with before/during/after execution
/// hooks will be implemented in Phase 3.
///
/// # Examples
///
/// ```rust
/// use airssys_osl::framework::pipeline::MiddlewarePipeline;
///
/// # #[tokio::main]
/// # async fn main() -> airssys_osl::core::result::OSResult<()> {
/// let mut pipeline = MiddlewarePipeline::new();
/// pipeline.initialize_all().await?;
/// assert!(pipeline.is_initialized());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct MiddlewarePipeline {
    /// Whether the pipeline has been initialized
    initialized: bool,
    /// Middleware names for tracking
    middleware_names: Vec<String>,
}

impl MiddlewarePipeline {
    /// Create a new middleware pipeline.
    pub fn new() -> Self {
        Self {
            initialized: false,
            middleware_names: Vec::new(),
        }
    }

    /// Add a middleware to the pipeline.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the middleware to add
    ///
    /// # Phase 2 Note
    ///
    /// Currently tracks middleware names. Phase 3 will store actual middleware
    /// instances and handle initialization.
    pub fn add_middleware(&mut self, name: String) {
        self.middleware_names.push(name);
    }

    /// Initialize all middleware in the pipeline.
    ///
    /// # Phase 2 Note
    ///
    /// Sets the initialized flag. Phase 3 will call actual middleware
    /// initialization methods.
    pub async fn initialize_all(&mut self) -> crate::core::result::OSResult<()> {
        self.initialized = true;
        Ok(())
    }

    /// Shutdown all middleware in the pipeline.
    ///
    /// # Phase 2 Note
    ///
    /// Clears the initialized flag. Phase 3 will call actual middleware
    /// shutdown methods.
    pub async fn shutdown_all(&mut self) -> crate::core::result::OSResult<()> {
        self.initialized = false;
        Ok(())
    }

    /// Get the number of middleware in the pipeline.
    pub fn middleware_count(&self) -> usize {
        self.middleware_names.len()
    }

    /// Check if the pipeline is initialized.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get the names of all middleware in the pipeline.
    pub fn middleware_names(&self) -> Vec<String> {
        self.middleware_names.clone()
    }
}

impl Default for MiddlewarePipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)] // Allow in tests for clarity

    use super::*;

    #[tokio::test]
    async fn test_pipeline_creation() {
        let pipeline = MiddlewarePipeline::new();
        assert_eq!(pipeline.middleware_count(), 0);
        assert!(!pipeline.is_initialized());
    }

    #[tokio::test]
    async fn test_initialize_all() {
        let mut pipeline = MiddlewarePipeline::new();
        let result = pipeline.initialize_all().await;
        assert!(result.is_ok());
        assert!(pipeline.is_initialized());
    }

    #[tokio::test]
    async fn test_shutdown_all() {
        let mut pipeline = MiddlewarePipeline::new();
        pipeline.initialize_all().await.expect("Initialize should succeed");
        
        let result = pipeline.shutdown_all().await;
        assert!(result.is_ok());
        assert!(!pipeline.is_initialized());
    }

    #[tokio::test]
    async fn test_middleware_names_empty() {
        let pipeline = MiddlewarePipeline::new();
        let names = pipeline.middleware_names();
        assert!(names.is_empty());
    }

    #[tokio::test]
    async fn test_add_middleware() {
        let mut pipeline = MiddlewarePipeline::new();
        pipeline.add_middleware("logger".to_string());
        pipeline.add_middleware("security".to_string());

        assert_eq!(pipeline.middleware_count(), 2);
        let names = pipeline.middleware_names();
        assert_eq!(names, vec!["logger", "security"]);
    }
}
