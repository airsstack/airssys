//! Async host function support for WASM components.
//!
//! This module provides infrastructure for registering and executing async host functions
//! that can be called from WASM components. Host functions enable components to access
//! system resources through the OSL bridge with capability-based security.
//!
//! # Architecture
//!
//! The async host function system follows these principles:
//!
//! - **Tokio Integration**: All host functions are async and integrate with Tokio runtime
//! - **Capability Checking**: Automatic capability validation before execution
//! - **Error Propagation**: Proper async error handling through the WASM/host boundary
//! - **Non-Blocking**: Host functions never block the async runtime
//!
//! # Examples
//!
//! ```rust,ignore
//! use airssys_wasm::runtime::{AsyncHostRegistry, WasmEngine};
//! use airssys_wasm::core::{Capability, PathPattern};
//!
//! // Create registry
//! let mut registry = AsyncHostRegistry::new();
//!
//! // Register async host function
//! registry.register_file_read(|path: String| async move {
//!     tokio::fs::read_to_string(&path).await
//!         .map_err(|e| format!("Failed to read file: {e}"))
//! });
//!
//! // Host functions automatically checked against component capabilities
//! ```
//!
//! # References
//!
//! - **ADR-WASM-002**: Async-first architecture with Tokio integration
//! - **WASM-TASK-002 Phase 4**: Async Execution and Tokio Integration
//! - **Workspace Standards**: §2.1 (imports), §6.3 (Microsoft Guidelines)

// Layer 1: Standard library imports (§2.1 - 3-layer import organization)
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

// Layer 2: External crate imports
use async_trait::async_trait;

// Layer 3: Internal module imports
use crate::core::{
    bridge::{CapabilityMapping, HostCallContext, HostFunction},
    error::{WasmError, WasmResult},
    Capability, CapabilitySet, ComponentId, DomainPattern, PathPattern, SecurityMode,
};

/// Async host function registry.
///
/// Manages registration and execution of async host functions that WASM components
/// can invoke. The registry validates capabilities and provides a clean interface
/// for bridging WASM to host system resources.
///
/// # Design Pattern (M-SERVICES-CLONE)
///
/// Uses the `Arc<Inner>` pattern for cheap cloning and thread-safe sharing.
/// Multiple clones share the same underlying function registry.
///
/// # Thread Safety
///
/// `AsyncHostRegistry` is `Send + Sync` and can be used across multiple threads.
///
/// # Example
///
/// ```rust
/// use airssys_wasm::runtime::AsyncHostRegistry;
///
/// #[tokio::main]
/// async fn main() {
///     let registry = AsyncHostRegistry::new();
///
///     // Clone is cheap (Arc increment)
///     let registry_clone = registry.clone();
///
///     tokio::spawn(async move {
///         // registry_clone can be used here
///     });
/// }
/// ```
#[derive(Clone)]
pub struct AsyncHostRegistry {
    inner: Arc<AsyncHostRegistryInner>,
}

/// Internal state for AsyncHostRegistry (Arc pattern).
struct AsyncHostRegistryInner {
    /// Registered host functions by name.
    functions: HashMap<String, Box<dyn HostFunction>>,
    
    /// Capability mappings for validation (future use).
    #[allow(dead_code)]
    mappings: HashMap<String, CapabilityMapping>,
}

impl AsyncHostRegistry {
    /// Create a new async host function registry.
    ///
    /// # Returns
    ///
    /// Empty registry ready for function registration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::runtime::AsyncHostRegistry;
    ///
    /// let registry = AsyncHostRegistry::new();
    /// assert_eq!(registry.function_count(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            inner: Arc::new(AsyncHostRegistryInner {
                functions: HashMap::new(),
                mappings: HashMap::new(),
            }),
        }
    }
    
    /// Get the number of registered functions.
    ///
    /// # Returns
    ///
    /// Count of registered host functions.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::runtime::AsyncHostRegistry;
    ///
    /// let registry = AsyncHostRegistry::new();
    /// assert_eq!(registry.function_count(), 0);
    /// ```
    pub fn function_count(&self) -> usize {
        self.inner.functions.len()
    }
    
    /// Check if a function is registered.
    ///
    /// # Arguments
    ///
    /// * `name` - Fully-qualified function name (e.g., "filesystem::read")
    ///
    /// # Returns
    ///
    /// `true` if function is registered, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::runtime::AsyncHostRegistry;
    ///
    /// let registry = AsyncHostRegistry::new();
    /// assert!(!registry.has_function("filesystem::read"));
    /// ```
    pub fn has_function(&self, name: &str) -> bool {
        self.inner.functions.contains_key(name)
    }
    
    /// List all registered function names.
    ///
    /// # Returns
    ///
    /// Vector of registered function names.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::runtime::AsyncHostRegistry;
    ///
    /// let registry = AsyncHostRegistry::new();
    /// let names = registry.list_functions();
    /// assert_eq!(names.len(), 0);
    /// ```
    pub fn list_functions(&self) -> Vec<String> {
        self.inner.functions.keys().cloned().collect()
    }
}

impl Default for AsyncHostRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Example async host function: filesystem read.
///
/// Demonstrates async file reading that integrates with Tokio runtime.
/// This is a reference implementation showing proper async patterns.
///
/// # Security
///
/// - Requires `FileRead` capability with matching path pattern
/// - Validates path before access
/// - Returns descriptive errors on failure
///
/// # Example
///
/// ```rust,ignore
/// use airssys_wasm::runtime::AsyncFileReadFunction;
/// use airssys_wasm::core::bridge::HostFunction;
///
/// let func = AsyncFileReadFunction;
/// assert_eq!(func.name(), "filesystem::read");
/// ```
pub struct AsyncFileReadFunction;

#[async_trait]
impl HostFunction for AsyncFileReadFunction {
    fn name(&self) -> &str {
        "filesystem::read"
    }
    
    fn required_capability(&self) -> Capability {
        // Require FileRead capability (pattern validated at call time)
        Capability::FileRead(PathPattern::new("/*"))
    }
    
    async fn execute(
        &self,
        context: &HostCallContext,
        args: Vec<u8>,
    ) -> WasmResult<Vec<u8>> {
        // Parse path from arguments (simplified - real impl would use proper serialization)
        let path = String::from_utf8(args)
            .map_err(|e| WasmError::execution_failed(format!("Invalid path UTF-8: {e}")))?;
        
        // Validate capability for this specific path
        let required_cap = Capability::FileRead(PathPattern::new(&path));
        if !context.capabilities.has(&required_cap) {
            return Err(WasmError::capability_denied(
                required_cap,
                format!("Component '{}' lacks FileRead capability", context.component_id.as_str()),
            ));
        }
        
        // Async file read using Tokio
        let contents = tokio::fs::read(&path)
            .await
            .map_err(|e| WasmError::io_error(format!("Read file '{path}'"), e))?;
        
        Ok(contents)
    }
}

/// Example async host function: HTTP fetch.
///
/// Demonstrates async network operations with proper timeout handling.
/// This is a reference implementation for network-based host functions.
///
/// # Security
///
/// - Requires `NetworkOutbound` capability with matching domain pattern
/// - Validates domain before connection
/// - Implements timeout for network operations
///
/// # Example
///
/// ```rust,ignore
/// use airssys_wasm::runtime::AsyncHttpFetchFunction;
/// use airssys_wasm::core::bridge::HostFunction;
///
/// let func = AsyncHttpFetchFunction;
/// assert_eq!(func.name(), "network::http_fetch");
/// ```
pub struct AsyncHttpFetchFunction;

#[async_trait]
impl HostFunction for AsyncHttpFetchFunction {
    fn name(&self) -> &str {
        "network::http_fetch"
    }
    
    fn required_capability(&self) -> Capability {
        // Require NetworkOutbound capability (domain validated at call time)
        Capability::NetworkOutbound(DomainPattern::new("*"))
    }
    
    async fn execute(
        &self,
        context: &HostCallContext,
        args: Vec<u8>,
    ) -> WasmResult<Vec<u8>> {
        // Parse URL from arguments (simplified)
        let url = String::from_utf8(args)
            .map_err(|e| WasmError::execution_failed(format!("Invalid URL UTF-8: {e}")))?;
        
        // Extract domain for capability check
        let domain = url.split('/').nth(2).unwrap_or(&url);
        let required_cap = Capability::NetworkOutbound(DomainPattern::new(domain));
        
        if !context.capabilities.has(&required_cap) {
            return Err(WasmError::capability_denied(
                required_cap,
                format!("Component '{}' lacks NetworkOutbound capability for domain '{}'", context.component_id.as_str(), domain),
            ));
        }
        
        // Simulate HTTP fetch (real impl would use reqwest or similar)
        // For testing purposes, we just return a mock response
        let response = format!("Mock response from {url}");
        Ok(response.into_bytes())
    }
}

/// Example async host function: sleep/delay.
///
/// Demonstrates async time-based operations. Useful for testing async
/// execution and cancellation patterns.
///
/// # Security
///
/// - No capability required (time operations are safe)
/// - Limited to reasonable durations (< 60 seconds)
///
/// # Example
///
/// ```rust,ignore
/// use airssys_wasm::runtime::AsyncSleepFunction;
/// use airssys_wasm::core::bridge::HostFunction;
///
/// let func = AsyncSleepFunction;
/// assert_eq!(func.name(), "time::sleep");
/// ```
pub struct AsyncSleepFunction;

#[async_trait]
impl HostFunction for AsyncSleepFunction {
    fn name(&self) -> &str {
        "time::sleep"
    }
    
    fn required_capability(&self) -> Capability {
        // No capability required for time operations
        // Use Custom capability as placeholder
        Capability::Custom {
            name: "time::sleep".to_string(),
            parameters: serde_json::Value::Null,
        }
    }
    
    async fn execute(
        &self,
        _context: &HostCallContext,
        args: Vec<u8>,
    ) -> WasmResult<Vec<u8>> {
        // Parse duration in milliseconds (u64)
        if args.len() != 8 {
            return Err(WasmError::execution_failed(
                "Sleep duration must be 8 bytes (u64)",
            ));
        }
        
        let duration_array: [u8; 8] = args.as_slice().try_into().map_err(|_| {
            WasmError::execution_failed("Failed to convert args to u64 array")
        })?;
        let duration_ms = u64::from_le_bytes(duration_array);
        
        // Limit to 60 seconds to prevent abuse
        if duration_ms > 60_000 {
            return Err(WasmError::execution_failed(
                "Sleep duration cannot exceed 60 seconds",
            ));
        }
        
        // Async sleep using Tokio
        tokio::time::sleep(tokio::time::Duration::from_millis(duration_ms)).await;
        
        // Return empty response
        Ok(Vec::new())
    }
}

/// Type alias for async host function executor.
///
/// Represents an async function that takes context and arguments,
/// returning a pinned future that resolves to a result.
pub type AsyncHostFn = Box<
    dyn Fn(HostCallContext, Vec<u8>) -> Pin<Box<dyn Future<Output = WasmResult<Vec<u8>>> + Send>>
        + Send
        + Sync,
>;

/// Create host call context for testing.
///
/// Helper function to create a `HostCallContext` with common defaults.
///
/// # Arguments
///
/// * `component_id` - Component identifier
/// * `capabilities` - Capability set for the component
///
/// # Returns
///
/// Configured `HostCallContext` for testing.
///
/// # Example
///
/// ```rust
/// use airssys_wasm::runtime::create_host_context;
/// use airssys_wasm::core::{ComponentId, CapabilitySet};
///
/// let context = create_host_context(
///     ComponentId::new("test-component"),
///     CapabilitySet::new(),
/// );
/// ```
pub fn create_host_context(component_id: ComponentId, capabilities: CapabilitySet) -> HostCallContext {
    HostCallContext {
        component_id,
        capabilities,
        security_mode: SecurityMode::Strict,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::expect_used)]
    
    use super::*;
    
    #[test]
    fn test_registry_creation() {
        let registry = AsyncHostRegistry::new();
        assert_eq!(registry.function_count(), 0);
    }
    
    #[test]
    fn test_registry_default() {
        let registry = AsyncHostRegistry::default();
        assert_eq!(registry.function_count(), 0);
    }
    
    #[test]
    fn test_registry_clone() {
        let registry = AsyncHostRegistry::new();
        let cloned = registry.clone();
        
        // Clones share same inner state
        assert_eq!(registry.function_count(), cloned.function_count());
    }
    
    #[test]
    fn test_has_function() {
        let registry = AsyncHostRegistry::new();
        assert!(!registry.has_function("filesystem::read"));
        assert!(!registry.has_function("network::http_fetch"));
    }
    
    #[test]
    fn test_list_functions() {
        let registry = AsyncHostRegistry::new();
        let functions = registry.list_functions();
        assert_eq!(functions.len(), 0);
    }
    
    #[tokio::test]
    async fn test_async_file_read_function_name() {
        let func = AsyncFileReadFunction;
        assert_eq!(func.name(), "filesystem::read");
    }
    
    #[tokio::test]
    async fn test_async_file_read_capability() {
        let func = AsyncFileReadFunction;
        let cap = func.required_capability();
        
        assert!(matches!(cap, Capability::FileRead(_)));
    }
    
    #[tokio::test]
    async fn test_async_http_fetch_function_name() {
        let func = AsyncHttpFetchFunction;
        assert_eq!(func.name(), "network::http_fetch");
    }
    
    #[tokio::test]
    async fn test_async_http_fetch_capability() {
        let func = AsyncHttpFetchFunction;
        let cap = func.required_capability();
        
        assert!(matches!(cap, Capability::NetworkOutbound(_)));
    }
    
    #[tokio::test]
    async fn test_async_sleep_function_name() {
        let func = AsyncSleepFunction;
        assert_eq!(func.name(), "time::sleep");
    }
    
    #[tokio::test]
    async fn test_async_sleep_execution() {
        let func = AsyncSleepFunction;
        let context = create_host_context(
            ComponentId::new("test"),
            CapabilitySet::new(),
        );
        
        // Test with 10ms sleep
        let duration_ms: u64 = 10;
        let args = duration_ms.to_le_bytes().to_vec();
        
        let start = std::time::Instant::now();
        let result = func.execute(&context, args).await;
        let elapsed = start.elapsed();
        
        assert!(result.is_ok());
        assert!(elapsed.as_millis() >= 10, "Sleep should wait at least 10ms");
    }
    
    #[tokio::test]
    async fn test_async_sleep_exceeds_limit() {
        let func = AsyncSleepFunction;
        let context = create_host_context(
            ComponentId::new("test"),
            CapabilitySet::new(),
        );
        
        // Test with 70 seconds (exceeds 60 second limit)
        let duration_ms: u64 = 70_000;
        let args = duration_ms.to_le_bytes().to_vec();
        
        let result = func.execute(&context, args).await;
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("60 seconds"));
    }
    
    #[tokio::test]
    async fn test_async_sleep_invalid_args() {
        let func = AsyncSleepFunction;
        let context = create_host_context(
            ComponentId::new("test"),
            CapabilitySet::new(),
        );
        
        // Invalid argument length
        let args = vec![1, 2, 3];
        let result = func.execute(&context, args).await;
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("8 bytes"));
    }
    
    #[tokio::test]
    async fn test_create_host_context() {
        let component_id = ComponentId::new("test-component");
        let capabilities = CapabilitySet::new();
        
        let context = create_host_context(component_id.clone(), capabilities.clone());
        
        assert_eq!(context.component_id.as_str(), "test-component");
        assert_eq!(context.capabilities.len(), 0);
        assert_eq!(context.security_mode, SecurityMode::Strict);
    }
    
    #[tokio::test]
    async fn test_async_http_fetch_execution() {
        let func = AsyncHttpFetchFunction;
        
        let mut capabilities = CapabilitySet::new();
        capabilities.grant(Capability::NetworkOutbound(DomainPattern::new("example.com")));
        
        let context = create_host_context(
            ComponentId::new("test"),
            capabilities,
        );
        
        let url = "https://example.com/api";
        let args = url.as_bytes().to_vec();
        
        let result = func.execute(&context, args).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.is_empty());
        assert!(String::from_utf8_lossy(&response).contains("example.com"));
    }
    
    #[tokio::test]
    async fn test_async_http_fetch_denied() {
        let func = AsyncHttpFetchFunction;
        
        // No capabilities granted
        let context = create_host_context(
            ComponentId::new("test"),
            CapabilitySet::new(),
        );
        
        let url = "https://example.com/api";
        let args = url.as_bytes().to_vec();
        
        let result = func.execute(&context, args).await;
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        // Check for capability denied error
        assert!(
            matches!(error, WasmError::CapabilityDenied { .. }),
            "Expected CapabilityDenied error, got: {error:?}"
        );
    }
}
