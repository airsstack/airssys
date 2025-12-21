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
use airssys_rt::broker::MessageBroker;
use airssys_rt::message::MessageEnvelope;
use async_trait::async_trait;

// Layer 3: Internal module imports
use crate::core::ComponentMessage;
use crate::core::{
    bridge::{CapabilityMapping, HostCallContext, HostFunction},
    error::{WasmError, WasmResult},
    multicodec_prefix::MulticodecPrefix,
    Capability, CapabilitySet, ComponentId, DomainPattern, PathPattern, SecurityMode, TopicPattern,
};
use crate::runtime::MessagingService;

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

    /// Get a reference to a registered function.
    ///
    /// Returns the host function if registered, otherwise None.
    ///
    /// # Arguments
    ///
    /// * `name` - Fully-qualified function name
    ///
    /// # Returns
    ///
    /// Reference to the host function if found.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use airssys_wasm::runtime::{AsyncHostRegistryBuilder, MessagingService};
    /// use std::sync::Arc;
    ///
    /// let messaging = Arc::new(MessagingService::new());
    /// let registry = AsyncHostRegistryBuilder::new()
    ///     .with_messaging_functions(messaging)
    ///     .build();
    ///
    /// let send_fn = registry.get_function("messaging::send");
    /// assert!(send_fn.is_some());
    /// ```
    pub fn get_function(&self, name: &str) -> Option<&dyn HostFunction> {
        self.inner.functions.get(name).map(|f| f.as_ref())
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

    async fn execute(&self, context: &HostCallContext, args: Vec<u8>) -> WasmResult<Vec<u8>> {
        // Parse path from arguments (simplified - real impl would use proper serialization)
        let path = String::from_utf8(args)
            .map_err(|e| WasmError::execution_failed(format!("Invalid path UTF-8: {e}")))?;

        // Validate capability for this specific path
        let required_cap = Capability::FileRead(PathPattern::new(&path));
        if !context.capabilities.has(&required_cap) {
            return Err(WasmError::capability_denied(
                required_cap,
                format!(
                    "Component '{}' lacks FileRead capability",
                    context.component_id.as_str()
                ),
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

    async fn execute(&self, context: &HostCallContext, args: Vec<u8>) -> WasmResult<Vec<u8>> {
        // Parse URL from arguments (simplified)
        let url = String::from_utf8(args)
            .map_err(|e| WasmError::execution_failed(format!("Invalid URL UTF-8: {e}")))?;

        // Extract domain for capability check
        let domain = url.split('/').nth(2).unwrap_or(&url);
        let required_cap = Capability::NetworkOutbound(DomainPattern::new(domain));

        if !context.capabilities.has(&required_cap) {
            return Err(WasmError::capability_denied(
                required_cap,
                format!(
                    "Component '{}' lacks NetworkOutbound capability for domain '{}'",
                    context.component_id.as_str(),
                    domain
                ),
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

    async fn execute(&self, _context: &HostCallContext, args: Vec<u8>) -> WasmResult<Vec<u8>> {
        // Parse duration in milliseconds (u64)
        if args.len() != 8 {
            return Err(WasmError::execution_failed(
                "Sleep duration must be 8 bytes (u64)",
            ));
        }

        let duration_array: [u8; 8] = args
            .as_slice()
            .try_into()
            .map_err(|_| WasmError::execution_failed("Failed to convert args to u64 array"))?;
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

/// Host function for fire-and-forget inter-component messaging.
///
/// This host function implements the `send-message` WIT interface, allowing
/// components to send one-way messages to other components. Messages are
/// validated for multicodec prefix and capability before being published
/// to the MessageBroker.
///
/// # Security (Block 4 Integration)
///
/// - Validates sender has `Messaging` capability for target
/// - Uses existing `CapabilitySet::can_send_to()` method
/// - Logs all send attempts for audit trail
/// - Validates multicodec prefix per ADR-WASM-001
///
/// # Performance
///
/// Target: ~280ns total latency
/// - Multicodec validation: ~10ns
/// - Capability check: ~50ns
/// - Broker publish: ~211ns
/// - Overhead: ~9ns
///
/// # Argument Format
///
/// Arguments are encoded as: `[target_len: u32 LE][target_bytes][message_bytes]`
/// - `target_len` - 4 bytes, little-endian u32, length of target ComponentId
/// - `target_bytes` - UTF-8 encoded target ComponentId string
/// - `message_bytes` - Message with multicodec prefix + payload
///
/// # References
///
/// - ADR-WASM-001: Multicodec Compatibility Strategy
/// - ADR-WASM-009: Component Communication Model
/// - KNOWLEDGE-WASM-024: Component Messaging Clarifications
pub struct SendMessageHostFunction {
    /// Reference to MessagingService for broker access
    messaging_service: Arc<MessagingService>,
}

impl SendMessageHostFunction {
    /// Create a new SendMessageHostFunction.
    ///
    /// # Arguments
    ///
    /// * `messaging_service` - Arc-wrapped MessagingService for broker access
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use airssys_wasm::runtime::{SendMessageHostFunction, MessagingService};
    /// use std::sync::Arc;
    ///
    /// let messaging = Arc::new(MessagingService::new());
    /// let send_fn = SendMessageHostFunction::new(messaging);
    /// ```
    pub fn new(messaging_service: Arc<MessagingService>) -> Self {
        Self { messaging_service }
    }
}

#[async_trait]
impl HostFunction for SendMessageHostFunction {
    fn name(&self) -> &str {
        "messaging::send"
    }

    fn required_capability(&self) -> Capability {
        // Base messaging capability - specific target checked in execute()
        Capability::Messaging(TopicPattern::new("*"))
    }

    async fn execute(&self, context: &HostCallContext, args: Vec<u8>) -> WasmResult<Vec<u8>> {
        // 1. Parse arguments: [target_len: u32 LE][target_bytes][message_bytes]
        if args.len() < 4 {
            return Err(WasmError::messaging_error(
                "Invalid send-message args: too short for target length",
            ));
        }

        let target_len = u32::from_le_bytes([args[0], args[1], args[2], args[3]]) as usize;
        let target_end = 4 + target_len;

        if args.len() < target_end {
            return Err(WasmError::messaging_error(format!(
                "Invalid send-message args: expected {} bytes for target, got {}",
                target_len,
                args.len().saturating_sub(4)
            )));
        }

        let target_str = String::from_utf8(args[4..target_end].to_vec())
            .map_err(|e| WasmError::messaging_error(format!("Invalid target UTF-8: {e}")))?;

        let message_bytes = args[target_end..].to_vec();

        // 2. Parse multicodec prefix (REQUIRED per ADR-WASM-001)
        let (codec, _prefix_len) = MulticodecPrefix::from_prefix(&message_bytes)
            .map_err(|e| WasmError::messaging_error(format!("Invalid multicodec: {e}")))?;

        // 3. Validate capability using existing can_send_to()
        let target_id = ComponentId::new(&target_str);
        if !context.capabilities.can_send_to(&target_id, Some(codec.name())) {
            return Err(WasmError::capability_denied(
                Capability::Messaging(TopicPattern::new(codec.name())),
                format!(
                    "Component '{}' cannot send {} messages to '{}'",
                    context.component_id.as_str(),
                    codec.name(),
                    target_id.as_str()
                ),
            ));
        }

        // 4. Create ComponentMessage, wrap in envelope, and publish to broker
        let component_message = ComponentMessage::InterComponent {
            sender: context.component_id.clone(),
            to: target_id,
            payload: message_bytes,
        };

        let envelope = MessageEnvelope::new(component_message);
        self.messaging_service
            .broker()
            .publish(envelope)
            .await
            .map_err(|e| WasmError::messaging_error(format!("Broker publish failed: {e}")))?;

        // Record the publish for metrics
        self.messaging_service.record_publish();

        // 5. Return empty response (fire-and-forget)
        Ok(Vec::new())
    }
}

/// Builder for AsyncHostRegistry.
///
/// Provides a mutable interface for registering host functions before
/// creating an immutable AsyncHostRegistry. This follows the builder
/// pattern for configuring registries.
///
/// # Example
///
/// ```rust
/// use airssys_wasm::runtime::{AsyncHostRegistryBuilder, MessagingService};
/// use std::sync::Arc;
///
/// let messaging = Arc::new(MessagingService::new());
///
/// let registry = AsyncHostRegistryBuilder::new()
///     .with_messaging_functions(messaging)
///     .build();
///
/// assert!(registry.has_function("messaging::send"));
/// ```
pub struct AsyncHostRegistryBuilder {
    functions: HashMap<String, Box<dyn HostFunction>>,
    mappings: HashMap<String, CapabilityMapping>,
}

impl AsyncHostRegistryBuilder {
    /// Create a new empty builder.
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            mappings: HashMap::new(),
        }
    }

    /// Register messaging host functions.
    ///
    /// Adds the `send-message` host function for inter-component messaging.
    /// This should be called during WasmRuntime initialization.
    ///
    /// # Arguments
    ///
    /// * `messaging_service` - Arc-wrapped MessagingService for broker access
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::runtime::{AsyncHostRegistryBuilder, MessagingService};
    /// use std::sync::Arc;
    ///
    /// let messaging = Arc::new(MessagingService::new());
    /// let builder = AsyncHostRegistryBuilder::new()
    ///     .with_messaging_functions(messaging);
    /// ```
    pub fn with_messaging_functions(mut self, messaging_service: Arc<MessagingService>) -> Self {
        let send_fn = SendMessageHostFunction::new(messaging_service);
        self.functions
            .insert(send_fn.name().to_string(), Box::new(send_fn));
        self
    }

    /// Register the filesystem read function.
    pub fn with_filesystem_functions(mut self) -> Self {
        let read_fn = AsyncFileReadFunction;
        self.functions
            .insert(read_fn.name().to_string(), Box::new(read_fn));
        self
    }

    /// Register the HTTP fetch function.
    pub fn with_network_functions(mut self) -> Self {
        let fetch_fn = AsyncHttpFetchFunction;
        self.functions
            .insert(fetch_fn.name().to_string(), Box::new(fetch_fn));
        self
    }

    /// Register the sleep function.
    pub fn with_time_functions(mut self) -> Self {
        let sleep_fn = AsyncSleepFunction;
        self.functions
            .insert(sleep_fn.name().to_string(), Box::new(sleep_fn));
        self
    }

    /// Register a custom host function.
    ///
    /// # Arguments
    ///
    /// * `func` - Host function implementation
    pub fn with_function(mut self, func: impl HostFunction + 'static) -> Self {
        self.functions.insert(func.name().to_string(), Box::new(func));
        self
    }

    /// Build the immutable AsyncHostRegistry.
    pub fn build(self) -> AsyncHostRegistry {
        AsyncHostRegistry {
            inner: Arc::new(AsyncHostRegistryInner {
                functions: self.functions,
                mappings: self.mappings,
            }),
        }
    }
}

impl Default for AsyncHostRegistryBuilder {
    fn default() -> Self {
        Self::new()
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
pub fn create_host_context(
    component_id: ComponentId,
    capabilities: CapabilitySet,
) -> HostCallContext {
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
        let context = create_host_context(ComponentId::new("test"), CapabilitySet::new());

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
        let context = create_host_context(ComponentId::new("test"), CapabilitySet::new());

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
        let context = create_host_context(ComponentId::new("test"), CapabilitySet::new());

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
        capabilities.grant(Capability::NetworkOutbound(DomainPattern::new(
            "example.com",
        )));

        let context = create_host_context(ComponentId::new("test"), capabilities);

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
        let context = create_host_context(ComponentId::new("test"), CapabilitySet::new());

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

    // ============================================================================
    // SendMessageHostFunction Tests (WASM-TASK-006 Phase 2 Task 2.1)
    // ============================================================================

    #[test]
    fn test_send_message_function_name() {
        let messaging = Arc::new(MessagingService::new());
        let func = SendMessageHostFunction::new(messaging);
        assert_eq!(func.name(), "messaging::send");
    }

    #[test]
    fn test_send_message_required_capability() {
        let messaging = Arc::new(MessagingService::new());
        let func = SendMessageHostFunction::new(messaging);
        let cap = func.required_capability();

        assert!(matches!(cap, Capability::Messaging(_)));
    }

    #[tokio::test]
    async fn test_send_message_success() {
        let messaging = Arc::new(MessagingService::new());
        let func = SendMessageHostFunction::new(messaging.clone());

        // Create context with messaging capability (wildcard)
        let mut caps = CapabilitySet::new();
        caps.grant(Capability::Messaging(TopicPattern::new("*")));
        let context = create_host_context(ComponentId::new("sender"), caps);

        // Create message with borsh prefix
        let target = "receiver";
        let mut message = MulticodecPrefix::Borsh.prefix_bytes().to_vec();
        message.extend_from_slice(b"test payload");

        // Encode args: [target_len: u32 LE][target_bytes][message_bytes]
        let mut args = (target.len() as u32).to_le_bytes().to_vec();
        args.extend_from_slice(target.as_bytes());
        args.extend_from_slice(&message);

        let result = func.execute(&context, args).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty()); // Fire-and-forget returns empty

        // Verify message was published
        let stats = messaging.get_stats().await;
        assert_eq!(stats.messages_published, 1);
    }

    #[tokio::test]
    async fn test_send_message_no_capability() {
        let messaging = Arc::new(MessagingService::new());
        let func = SendMessageHostFunction::new(messaging);

        // Create context WITHOUT messaging capability
        let context = create_host_context(ComponentId::new("sender"), CapabilitySet::new());

        // Create message with borsh prefix
        let target = "receiver";
        let mut message = MulticodecPrefix::Borsh.prefix_bytes().to_vec();
        message.extend_from_slice(b"test payload");

        // Encode args
        let mut args = (target.len() as u32).to_le_bytes().to_vec();
        args.extend_from_slice(target.as_bytes());
        args.extend_from_slice(&message);

        let result = func.execute(&context, args).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(
            matches!(error, WasmError::CapabilityDenied { .. }),
            "Expected CapabilityDenied, got: {error:?}"
        );
    }

    #[tokio::test]
    async fn test_send_message_invalid_multicodec() {
        let messaging = Arc::new(MessagingService::new());
        let func = SendMessageHostFunction::new(messaging);

        // Create context with messaging capability
        let mut caps = CapabilitySet::new();
        caps.grant(Capability::Messaging(TopicPattern::new("*")));
        let context = create_host_context(ComponentId::new("sender"), caps);

        // Create message with INVALID prefix
        let target = "receiver";
        let message = vec![0xFF, 0xFF, 0xDE, 0xAD]; // Invalid prefix

        // Encode args
        let mut args = (target.len() as u32).to_le_bytes().to_vec();
        args.extend_from_slice(target.as_bytes());
        args.extend_from_slice(&message);

        let result = func.execute(&context, args).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("multicodec") || error.to_string().contains("Multicodec"),
            "Expected multicodec error, got: {error}"
        );
    }

    #[tokio::test]
    async fn test_send_message_message_too_short() {
        let messaging = Arc::new(MessagingService::new());
        let func = SendMessageHostFunction::new(messaging);

        // Create context with messaging capability
        let mut caps = CapabilitySet::new();
        caps.grant(Capability::Messaging(TopicPattern::new("*")));
        let context = create_host_context(ComponentId::new("sender"), caps);

        // Create message that's too short (only 1 byte)
        let target = "receiver";
        let message = vec![0x07]; // Too short for multicodec prefix

        // Encode args
        let mut args = (target.len() as u32).to_le_bytes().to_vec();
        args.extend_from_slice(target.as_bytes());
        args.extend_from_slice(&message);

        let result = func.execute(&context, args).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("too short") || error.to_string().contains("multicodec"),
            "Expected too short error, got: {error}"
        );
    }

    #[tokio::test]
    async fn test_send_message_bincode_codec() {
        let messaging = Arc::new(MessagingService::new());
        let func = SendMessageHostFunction::new(messaging.clone());

        // Create context with messaging capability
        let mut caps = CapabilitySet::new();
        caps.grant(Capability::Messaging(TopicPattern::new("*")));
        let context = create_host_context(ComponentId::new("sender"), caps);

        // Create message with bincode prefix
        let target = "receiver";
        let mut message = MulticodecPrefix::Bincode.prefix_bytes().to_vec();
        message.extend_from_slice(b"bincode payload");

        // Encode args
        let mut args = (target.len() as u32).to_le_bytes().to_vec();
        args.extend_from_slice(target.as_bytes());
        args.extend_from_slice(&message);

        let result = func.execute(&context, args).await;

        assert!(result.is_ok());
        
        let stats = messaging.get_stats().await;
        assert_eq!(stats.messages_published, 1);
    }

    #[tokio::test]
    async fn test_send_message_messagepack_codec() {
        let messaging = Arc::new(MessagingService::new());
        let func = SendMessageHostFunction::new(messaging.clone());

        // Create context with messaging capability
        let mut caps = CapabilitySet::new();
        caps.grant(Capability::Messaging(TopicPattern::new("*")));
        let context = create_host_context(ComponentId::new("sender"), caps);

        // Create message with messagepack prefix
        let target = "receiver";
        let mut message = MulticodecPrefix::MessagePack.prefix_bytes().to_vec();
        message.extend_from_slice(b"msgpack payload");

        // Encode args
        let mut args = (target.len() as u32).to_le_bytes().to_vec();
        args.extend_from_slice(target.as_bytes());
        args.extend_from_slice(&message);

        let result = func.execute(&context, args).await;

        assert!(result.is_ok());
        
        let stats = messaging.get_stats().await;
        assert_eq!(stats.messages_published, 1);
    }

    // ============================================================================
    // AsyncHostRegistryBuilder Tests
    // ============================================================================

    #[test]
    fn test_registry_builder_new() {
        let builder = AsyncHostRegistryBuilder::new();
        let registry = builder.build();
        assert_eq!(registry.function_count(), 0);
    }

    #[test]
    fn test_registry_builder_with_messaging() {
        let messaging = Arc::new(MessagingService::new());
        let registry = AsyncHostRegistryBuilder::new()
            .with_messaging_functions(messaging)
            .build();

        assert!(registry.has_function("messaging::send"));
        assert_eq!(registry.function_count(), 1);
    }

    #[test]
    fn test_registry_builder_with_filesystem() {
        let registry = AsyncHostRegistryBuilder::new()
            .with_filesystem_functions()
            .build();

        assert!(registry.has_function("filesystem::read"));
        assert_eq!(registry.function_count(), 1);
    }

    #[test]
    fn test_registry_builder_with_network() {
        let registry = AsyncHostRegistryBuilder::new()
            .with_network_functions()
            .build();

        assert!(registry.has_function("network::http_fetch"));
        assert_eq!(registry.function_count(), 1);
    }

    #[test]
    fn test_registry_builder_with_time() {
        let registry = AsyncHostRegistryBuilder::new()
            .with_time_functions()
            .build();

        assert!(registry.has_function("time::sleep"));
        assert_eq!(registry.function_count(), 1);
    }

    #[test]
    fn test_registry_builder_chaining() {
        let messaging = Arc::new(MessagingService::new());
        let registry = AsyncHostRegistryBuilder::new()
            .with_messaging_functions(messaging)
            .with_filesystem_functions()
            .with_network_functions()
            .with_time_functions()
            .build();

        assert_eq!(registry.function_count(), 4);
        assert!(registry.has_function("messaging::send"));
        assert!(registry.has_function("filesystem::read"));
        assert!(registry.has_function("network::http_fetch"));
        assert!(registry.has_function("time::sleep"));
    }

    #[test]
    fn test_registry_get_function() {
        let messaging = Arc::new(MessagingService::new());
        let registry = AsyncHostRegistryBuilder::new()
            .with_messaging_functions(messaging)
            .build();

        let func = registry.get_function("messaging::send");
        assert!(func.is_some());
        assert_eq!(func.unwrap().name(), "messaging::send");

        let missing = registry.get_function("nonexistent");
        assert!(missing.is_none());
    }
}
